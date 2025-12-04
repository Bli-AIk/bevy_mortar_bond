//! Split terminal + gameplay mock example (Refactored for real MortarRuntime).
//!
//! The window is divided into two panes: a faux Unix terminal on the left and a
//! placeholder gameplay viewport on the right.
//!
//! This example demonstrates how to integrate `MortarDialoguePlugin` with a custom
//! UI system (the terminal typewriter) by using a proxy entity to capture text.
//!
//! Sprite from https://opengameart.org/content/animated-rogue
#[path = "utils/live_terminal.rs"]
mod live_terminal;
#[path = "utils/rogue_sprite.rs"]
mod rogue_sprite;
#[path = "utils/typewriter.rs"]
mod typewriter;

use bevy::{
    asset::AssetPlugin,
    ecs::message::{MessageReader, MessageWriter},
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
    },
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_mortar_bond::{
    MortarBoolean, MortarDialoguePlugin, MortarDialogueText, MortarEvent, MortarFunctions,
    MortarGameEvent, MortarPlugin, MortarRegistry, MortarRuntime, MortarTextTarget,
    mortar_functions,
};
use live_terminal::{
    ASSET_DIR, ChoiceButton, ChoicePanel, ChoicePanelFont, CursorBlink, DEFAULT_FILE,
    DIALOGUE_CHAR_SPEED, GameDialogueText, RogueAnimationEvent, RoguePreviewImage, TerminalMachine,
    animation_from_label, apply_animation_events, despawn_recursive, handle_keyboard_controls,
    handle_panel_focus, refresh_terminal_display, revert_animation_to_idle, setup_ui,
    tick_cursor_blink, update_focus_visuals,
};
use rogue_sprite::{RogueGender, RogueSprite, RogueSpritePlugin};
use std::{fs, path::Path, time::SystemTime};
use typewriter::{Typewriter, TypewriterPlugin, TypewriterState};

fn main() {
    App::new()
        .init_resource::<TerminalMachine>()
        .init_resource::<CursorBlink>()
        .add_message::<RogueAnimationEvent>()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Live Mortar Editor".into(),
                        resolution: WindowResolution::new(1200, 720),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: ASSET_DIR.into(),
                    ..default()
                }),
        )
        .add_plugins((
            TypewriterPlugin,
            RogueSpritePlugin,
            MortarPlugin,
            MortarDialoguePlugin,
        ))
        .init_resource::<LiveScriptSource>()
        .init_resource::<ScriptWatcher>()
        .add_systems(Startup, (setup_ui, setup_mortar_integration))
        .add_systems(
            Update,
            (
                tick_cursor_blink,
                handle_panel_focus,
                handle_keyboard_controls,
                handle_dialogue_input,
                handle_choice_buttons,
                sync_mortar_text_to_terminal,
                bridge_mortar_events,
                sync_choice_panel,
                monitor_script_changes,
            ),
        )
        .add_systems(
            Update,
            (
                update_dialogue_text_render,
                apply_animation_events,
                revert_animation_to_idle,
                refresh_terminal_display,
                update_focus_visuals,
            ),
        )
        .run();
}

/// A hidden entity used to receive processed text from MortarDialoguePlugin.
/// We read from this and feed the visible typewriter.
#[derive(Component)]
struct MortarTextProxy;

#[derive(MortarFunctions)]
struct TerminalFunctions;

#[mortar_functions]
impl TerminalFunctions {
    fn get_name() -> String {
        "Player".to_string()
    }

    fn set_gender(_is_female: MortarBoolean) {
        // Handled via events or variable state inspection if needed
    }
}

// --- Source Mapping & Highlighting Support ---

#[derive(Resource, Default)]
pub struct LiveScriptSource {
    // Stores (NodeName, Step)
    pub entries: Vec<(String, DialogueStep)>,
    pub last_modified: Option<SystemTime>,
}

impl LiveScriptSource {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(path)?;
        let modified = fs::metadata(path).and_then(|m| m.modified()).ok();
        let parsed = parse_script_contents(&contents);
        Ok(Self {
            entries: parsed.entries,
            last_modified: modified,
        })
    }

    /// Heuristic to find the source line number for the current runtime state.
    /// This matches the parsed entries for the current node against the runtime's text index.
    pub fn get_highlight_line(&self, runtime: &MortarRuntime) -> Option<usize> {
        let state = runtime.active_dialogue.as_ref()?;
        let current_node = &state.current_node;
        
        let mut current_text_count = 0;
        
        for (node_name, entry) in &self.entries {
            if node_name != current_node {
                continue;
            }

            match entry {
                DialogueStep::Line(line) => {
                    if current_text_count == state.text_index {
                         return Some(line.line_number);
                    }
                    current_text_count += 1;
                }
                DialogueStep::Choice(_) => {
                    // If we are waiting for a choice, highlighting the choice block 
                    // (or the last text) might be nice, but for now we just skip.
                }
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct DialogueLine {
    text: String,
    events: Vec<LineEvent>,
    line_number: usize,
}

#[derive(Clone)]
pub struct ChoiceOption {
    label: String,
    target: ChoiceTarget,
}

#[derive(Clone)]
enum ChoiceTarget {
    Return,
    Node(String),
}

#[derive(Clone)]
pub enum DialogueStep {
    Line(DialogueLine),
    Choice(Vec<ChoiceOption>),
}

#[derive(Clone)]
pub struct LineEvent {
    trigger_index: usize,
    action: LineEventAction,
}

#[derive(Clone)]
pub enum LineEventAction {
    PlayAnim(String),
}

struct ParsedScript {
    is_female: bool,
    entries: Vec<(String, DialogueStep)>,
}

fn parse_script_contents(contents: &str) -> ParsedScript {
    // We no longer need to extract is_female because we map ALL text lines 
    // to match the runtime's physical index structure.
    let entries = collect_entries(contents);
    ParsedScript { is_female: false, entries }
}

fn collect_entries(contents: &str) -> Vec<(String, DialogueStep)> {
    let mut entries = Vec::new();
    let mut lines = contents.lines().enumerate().peekable();
    let mut current_node = String::new();

    while let Some((line_number, line)) = lines.next() {
        let trimmed = line.trim();

        if trimmed.starts_with("node ") {
            // node Start {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                current_node = parts[1].trim_matches('{').to_string();
            }
            continue;
        }

        // We deliberately IGNORE if/else blocks here.
        // The Mortar runtime's `text_index` counts ALL text nodes in the compiled asset,
        // regardless of whether they are executed or skipped at runtime.
        // To ensure our index matches the runtime's index, we must collect ALL text lines found in the source.

        if let Some(text) = parse_text_line(trimmed) {
            entries.push((current_node.clone(), DialogueStep::Line(DialogueLine {
                text,
                events: Vec::new(),
                line_number: line_number + 1,
            })));
            continue;
        }
        
        if trimmed.starts_with("with events") {
            // Attach events to the last entry if it belongs to the same node
            if let Some((last_node, last_step)) = entries.last_mut()
                && *last_node == current_node
                && matches!(last_step, DialogueStep::Line(_))
            {
                if let DialogueStep::Line(line) = last_step {
                    collect_event_entries(&mut lines, line);
                }
            } else {
                consume_event_block(&mut lines);
            }
            continue;
        }
        
        if trimmed.starts_with("choice") {
            let options = collect_choice_entries(&mut lines);
            if !options.is_empty() {
                entries.push((current_node.clone(), DialogueStep::Choice(options)));
            }
        }
    }
    entries
}

// Removed unused ConditionalContext struct and impl
fn parse_text_line(line: &str) -> Option<String> {
    if !line.starts_with("text:") {
        return None;
    }
    let mut parts = line.splitn(2, '"');
    parts.next()?;
    let rest = parts.next()?;
    let mut segments = rest.splitn(2, '"');
    let content = segments.next()?;
    Some(content.to_string())
}

fn collect_event_entries(
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>,
    line: &mut DialogueLine,
) {
    while let Some((_, line_text)) = lines.next() {
        let trimmed = line_text.trim();
        if trimmed.starts_with(']') {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if let Some(event) = parse_event_line(trimmed) {
            line.events.push(event);
        }
    }
}

fn consume_event_block(lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>) {
    while let Some((_, line)) = lines.next() {
        if line.trim().starts_with(']') {
            break;
        }
    }
}

fn collect_choice_entries(
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>,
) -> Vec<ChoiceOption> {
    let mut options = Vec::new();
    while let Some((_, line_text)) = lines.next() {
        let trimmed = line_text.trim();
        if trimmed.starts_with(']') {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if let Some(option) = parse_choice_line(trimmed) {
            options.push(option);
        }
    }
    options
}

fn parse_event_line(line: &str) -> Option<LineEvent> {
    let cleaned = line.trim().trim_end_matches(',');
    if cleaned.is_empty() {
        return None;
    }
    let mut parts = cleaned.splitn(2, ',');
    let index_str = parts.next()?.trim();
    let trigger_index = index_str.parse::<usize>().ok()?;
    let action = parts.next()?.trim();
    if let Some(name) = action.strip_prefix("play_anim(") {
        let label = name.trim().trim_matches(|c| c == '"' || c == ')');
        return Some(LineEvent {
            trigger_index,
            action: LineEventAction::PlayAnim(label.to_string()),
        });
    }
    None
}

fn parse_choice_line(line: &str) -> Option<ChoiceOption> {
    let cleaned = line.trim().trim_end_matches(',');
    let mut parts = cleaned.splitn(2, "->");
    let label_part = parts.next()?.trim().trim_matches('"');
    if label_part.is_empty() {
        return None;
    }
    let target_part = parts.next()?.trim().trim_end_matches(',');
    let target = if target_part.eq_ignore_ascii_case("return") {
        ChoiceTarget::Return
    } else {
        ChoiceTarget::Node(target_part.to_string())
    };
    Some(ChoiceOption {
        label: label_part.to_string(),
        target,
    })
}

#[derive(Resource)]
struct ScriptWatcher {
    timer: Timer,
}

impl Default for ScriptWatcher {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

fn monitor_script_changes(
    time: Res<Time>,
    mut watcher: ResMut<ScriptWatcher>,
    mut source: ResMut<LiveScriptSource>,
) {
    if !watcher.timer.tick(time.delta()).just_finished() {
        return;
    }
    let path = live_terminal::live_root_path().join(live_terminal::DEFAULT_FILE);
    let modified = fs::metadata(&path)
        .ok()
        .and_then(|meta| meta.modified().ok());

    if source.last_modified != modified {
        if let Ok(new_source) = LiveScriptSource::from_path(&path) {
            *source = new_source;
        }
    }
}

fn setup_mortar_integration(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    mut runtime: ResMut<MortarRuntime>,
    mut source: ResMut<LiveScriptSource>,
) {
    // Register functions
    TerminalFunctions::bind_functions(&mut runtime.functions);

    // Load the live example script
    let path = format!("live/{}", DEFAULT_FILE);
    let handle = asset_server.load(&path);
    registry.register(path.clone(), handle);

    // Parse initial source map
    let fs_path = live_terminal::live_root_path().join(DEFAULT_FILE);
    if let Ok(initial_source) = LiveScriptSource::from_path(&fs_path) {
        *source = initial_source;
    }

    // Create a hidden proxy entity to receive Mortar text updates.
    // MortarDialoguePlugin expects a Text component to write into.
    commands.spawn((
        Text::new(""),
        MortarTextTarget,
        MortarDialogueText::default(),
        MortarTextProxy,
        Visibility::Hidden,
    ));

    // Start the dialogue
    events.write(MortarEvent::StartNode {
        path,
        node: "Start".to_string(),
    });
}

fn handle_dialogue_input(
    mut inputs: MessageReader<KeyboardInput>,
    machine: Res<TerminalMachine>,
    runtime: Res<MortarRuntime>,
    mut events: MessageWriter<MortarEvent>,
    mut text_query: Query<&mut Typewriter, With<GameDialogueText>>,
) {
    if machine.focused {
        return;
    }

    // Don't process input if we have active choices
    if let Some(state) = &runtime.active_dialogue {
        if state.has_choices() {
            return;
        }
    }

    let Ok(mut typewriter) = text_query.single_mut() else {
        return;
    };

    for input in inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        if input.key_code == KeyCode::KeyZ {
            if typewriter.is_playing() {
                // Skip typing effect
                typewriter.current_text = typewriter.source_text.clone();
                typewriter.current_char_index = typewriter.source_text.chars().count();
                typewriter.state = TypewriterState::Finished;
            } else if typewriter.state == TypewriterState::Finished
                || typewriter.state == TypewriterState::Idle
            {
                // Request next text from Mortar
                events.write(MortarEvent::NextText);
            }
        }
    }
}

/// Watches the hidden proxy entity. When MortarDialoguePlugin updates it,
/// we forward the text to the visible Typewriter.
fn sync_mortar_text_to_terminal(
    proxy_query: Query<&MortarDialogueText, (With<MortarTextProxy>, Changed<MortarDialogueText>)>,
    mut terminal_query: Query<&mut Typewriter, With<GameDialogueText>>,
    mut machine: ResMut<TerminalMachine>,
) {
    let Ok(mortar_text) = proxy_query.single() else {
        return;
    };
    let Ok(mut typewriter) = terminal_query.single_mut() else {
        return;
    };

    // Start typing the new text
    *typewriter = Typewriter::new(mortar_text.body.clone(), DIALOGUE_CHAR_SPEED);
    typewriter.play();

    // Force refresh terminal to update highlight line in the editor
    machine.dirty = true;
}

fn sync_choice_panel(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    mut panel_query: Query<
        (Entity, Option<&Children>, &mut Visibility, &ChoicePanelFont),
        With<ChoicePanel>,
    >,
    child_query: Query<&Children>,
    // Track if we've already built choices for this text index to avoid rebuilding every frame
    mut last_choice_index: Local<Option<(String, usize)>>,
) {
    let Ok((panel_entity, children, mut visibility, font)) = panel_query.single_mut() else {
        return;
    };

    let Some(state) = &runtime.active_dialogue else {
        *visibility = Visibility::Hidden;
        return;
    };

    // Check if we have choices active
    let Some(choices) = state.get_current_choices() else {
        *visibility = Visibility::Hidden;
        *last_choice_index = None;
        return;
    };

    // Avoid rebuilding if nothing changed
    let current_key = (state.current_node.clone(), state.text_index);
    if last_choice_index.as_ref() == Some(&current_key) && *visibility == Visibility::Visible {
        return;
    }
    *last_choice_index = Some(current_key);

    // Clear old choices
    if let Some(children) = children {
        for child in children.iter() {
            despawn_recursive(child, &mut commands, &child_query);
        }
    }

    *visibility = Visibility::Visible;

    commands.entity(panel_entity).with_children(|parent| {
        for (index, option) in choices.iter().enumerate() {
            parent
                .spawn((
                    Button,
                    ChoiceButton { index },
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.18, 0.3)),
                    BorderColor::all(Color::srgb(0.4, 0.5, 0.8)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(option.text.clone()),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));
                });
        }
    });
}

fn handle_choice_buttons(
    mut buttons: Query<(&Interaction, &ChoiceButton), (Changed<Interaction>, With<Button>)>,
    mut events: MessageWriter<MortarEvent>,
) {
    for (interaction, button) in &mut buttons {
        if *interaction == Interaction::Pressed {
            events.write(MortarEvent::SelectChoice {
                index: button.index,
            });
            events.write(MortarEvent::ConfirmChoice);
            break; // Only handle one click per frame
        }
    }
}

fn update_dialogue_text_render(
    mut query: Query<&mut Text, With<GameDialogueText>>,
    typewriter_query: Query<&Typewriter, (With<GameDialogueText>, Changed<Typewriter>)>,
) {
    if let Ok(typewriter) = typewriter_query.single() {
        if let Ok(mut text) = query.single_mut() {
            // Update the visible text component from the typewriter state
            **text = typewriter.current_text.clone();
        }
    }
}

/// Bridges MortarGameEvent (from script) to RogueAnimationEvent (for visuals)
fn bridge_mortar_events(
    mut mortar_events: MessageReader<MortarGameEvent>,
    mut anim_events: MessageWriter<RogueAnimationEvent>,
    mut preview: Query<&mut RogueSprite, With<RoguePreviewImage>>,
) {
    for event in mortar_events.read() {
        match event.name.as_str() {
            "play_anim" => {
                if let Some(anim_name) = event.args.first() {
                    if let Some(animation) = animation_from_label(anim_name) {
                        anim_events.write(RogueAnimationEvent { animation });
                    }
                }
            }
            "set_gender" => {
                if let Some(arg) = event.args.first() {
                    let is_female = arg.to_lowercase() == "true";
                    if let Ok(mut sprite) = preview.single_mut() {
                        sprite.gender = if is_female {
                            RogueGender::Female
                        } else {
                            RogueGender::Male
                        };
                    }
                }
            }
            _ => {
                // Handle other events if necessary
            }
        }
    }
}
