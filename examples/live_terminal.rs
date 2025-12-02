//! Split terminal + gameplay mock example.
//!
//! The window is divided into two panes: a faux Unix terminal on the left and a
//! placeholder gameplay viewport on the right. Click the terminal to capture
//! keyboard focus, run `bevim live_example`, and then edit the Mortar script in
//! a tiny vim-inspired editor. The right-hand view is left as a TODO hook for
//! real gameplay rendering.
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
    log::warn,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use live_terminal::{
    ASSET_DIR, ChoiceButton, ChoicePanel, ChoicePanelFont, CursorBlink, DEFAULT_FILE,
    DIALOGUE_CHAR_SPEED, DIALOGUE_FINISHED_LINE, DIALOGUE_PLACEHOLDER, GameDialogueText,
    RogueAnimationEvent, RoguePreviewImage, TerminalMachine, TerminalView, animation_from_label,
    apply_animation_events, despawn_recursive, handle_keyboard_controls, handle_panel_focus,
    live_root_path, refresh_terminal_display, revert_animation_to_idle, setup_ui,
    tick_cursor_blink, update_focus_visuals,
};
use rogue_sprite::{RogueGender, RogueSprite, RogueSpritePlugin};
use std::{fs, path::Path, time::SystemTime};
use typewriter::{Typewriter, TypewriterPlugin, TypewriterState};

fn main() {
    App::new()
        .init_resource::<TerminalMachine>()
        .init_resource::<CursorBlink>()
        .init_resource::<LiveDialogueData>()
        .init_resource::<LiveDialogueWatcher>()
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
        .add_plugins((TypewriterPlugin, RogueSpritePlugin))
        .add_systems(Startup, setup_ui)
        .add_systems(
            Update,
            (
                tick_cursor_blink,
                handle_panel_focus,
                handle_keyboard_controls,
                handle_dialogue_input,
                handle_choice_buttons,
                monitor_live_example_script,
                apply_gender_from_script,
                apply_pending_dialogue_text,
                sync_choice_panel,
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

#[derive(Component, Default)]
struct ActiveLineEvents {
    events: Vec<LineEvent>,
    next_event: usize,
}

impl ActiveLineEvents {
    fn from_events(events: Vec<LineEvent>) -> Self {
        Self {
            events,
            next_event: 0,
        }
    }
}

fn handle_dialogue_input(
    mut inputs: MessageReader<KeyboardInput>,
    machine: Res<TerminalMachine>,
    mut dialogue: ResMut<LiveDialogueData>,
    mut text_query: Query<&mut Typewriter, With<GameDialogueText>>,
) {
    if machine.focused {
        return;
    }
    let Ok(mut typewriter) = text_query.single_mut() else {
        return;
    };
    for input in inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        if dialogue.is_waiting_for_choice() {
            continue;
        }
        if input.key_code == KeyCode::KeyZ {
            if typewriter.is_playing() {
                typewriter.current_text = typewriter.source_text.clone();
                typewriter.current_char_index = typewriter.source_text.chars().count();
                typewriter.state = TypewriterState::Finished;
            } else if typewriter.state == TypewriterState::Finished
                || typewriter.state == TypewriterState::Idle
            {
                dialogue.advance_text();
            }
        }
    }
}

fn apply_gender_from_script(
    mut dialogue: ResMut<LiveDialogueData>,
    mut preview: Query<&mut RogueSprite, With<RoguePreviewImage>>,
) {
    if !dialogue.needs_gender_sync {
        return;
    }
    let Ok(mut sprite) = preview.single_mut() else {
        return;
    };
    let target = if dialogue.is_female {
        RogueGender::Female
    } else {
        RogueGender::Male
    };
    sprite.gender = target;
    dialogue.needs_gender_sync = false;
}

fn apply_pending_dialogue_text(
    mut commands: Commands,
    mut dialogue: ResMut<LiveDialogueData>,
    mut text_query: Query<(Entity, &mut Typewriter), With<GameDialogueText>>,
    mut machine: ResMut<TerminalMachine>,
) {
    if !dialogue.needs_text_update {
        return;
    }
    let Some(line) = dialogue.pending_line.take() else {
        dialogue.needs_text_update = false;
        return;
    };
    let Ok((entity, mut typewriter)) = text_query.single_mut() else {
        return;
    };
    let DialogueLine {
        text,
        events,
        line_number: _,
    } = line;
    let instant =
        events.is_empty() && (text == DIALOGUE_PLACEHOLDER || text == DIALOGUE_FINISHED_LINE);
    *typewriter = Typewriter::new(text.clone(), DIALOGUE_CHAR_SPEED);
    if instant {
        typewriter.current_text = text.clone();
        typewriter.current_char_index = text.chars().count();
        typewriter.state = TypewriterState::Finished;
    } else {
        typewriter.play();
    }
    if events.is_empty() {
        commands.entity(entity).remove::<ActiveLineEvents>();
    } else {
        commands
            .entity(entity)
            .insert(ActiveLineEvents::from_events(events));
    }
    if dialogue.take_highlight_dirty() && matches!(machine.view, TerminalView::Vim(_)) {
        machine.dirty = true;
    }
    dialogue.needs_text_update = false;
}

fn sync_choice_panel(
    mut commands: Commands,
    mut dialogue: ResMut<LiveDialogueData>,
    mut panel_query: Query<
        (Entity, Option<&Children>, &mut Visibility, &ChoicePanelFont),
        With<ChoicePanel>,
    >,
    child_query: Query<&Children>,
) {
    if !dialogue.take_choice_dirty() {
        return;
    }
    let Ok((panel_entity, children, mut visibility, font)) = panel_query.single_mut() else {
        return;
    };
    if let Some(children) = children {
        for child in children.iter() {
            despawn_recursive(child, &mut commands, &child_query);
        }
    }
    if let Some(options) = dialogue.pending_choices() {
        *visibility = Visibility::Visible;
        commands.entity(panel_entity).with_children(|parent| {
            for (index, option) in options.iter().enumerate() {
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
                            Text::new(option.label.clone()),
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
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn handle_choice_buttons(
    mut buttons: Query<(&Interaction, &ChoiceButton), (Changed<Interaction>, With<Button>)>,
    mut dialogue: ResMut<LiveDialogueData>,
) {
    if !dialogue.is_waiting_for_choice() {
        return;
    }
    for (interaction, button) in &mut buttons {
        if *interaction == Interaction::Pressed && dialogue.select_choice(button.index) {
            break;
        }
    }
}

fn update_dialogue_text_render(
    mut writer: MessageWriter<RogueAnimationEvent>,
    mut query: Query<
        (&Typewriter, &mut Text, Option<&mut ActiveLineEvents>),
        With<GameDialogueText>,
    >,
) {
    for (typewriter, mut text, events) in &mut query {
        **text = typewriter.current_text.clone();
        if let Some(mut active) = events {
            while let Some(event) = active.events.get(active.next_event) {
                let required = event.trigger_index.saturating_add(1);
                if typewriter.current_char_index >= required {
                    if let Some(animation) = animation_from_label(match &event.action {
                        LineEventAction::PlayAnim(label) => label,
                    }) {
                        writer.write(RogueAnimationEvent { animation });
                    }
                    active.next_event += 1;
                } else {
                    break;
                }
            }
        }
    }
}

fn monitor_live_example_script(
    time: Res<Time>,
    mut watcher: ResMut<LiveDialogueWatcher>,
    mut dialogue: ResMut<LiveDialogueData>,
) {
    if !watcher.timer.tick(time.delta()).just_finished() {
        return;
    }
    let path = live_root_path().join(DEFAULT_FILE);
    let modified = fs::metadata(&path)
        .ok()
        .and_then(|meta| meta.modified().ok());
    if dialogue.last_modified == modified {
        return;
    }
    match ParsedScript::from_path(&path) {
        Ok(parsed) => dialogue.apply_parsed(parsed, modified),
        Err(err) => warn!("无法解析 {}: {}", path.display(), err),
    }
}

#[derive(Resource)]
struct LiveDialogueWatcher {
    timer: Timer,
}

impl Default for LiveDialogueWatcher {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

#[derive(Clone)]
struct DialogueLine {
    text: String,
    events: Vec<LineEvent>,
    line_number: usize,
}

#[derive(Clone)]
struct ChoiceOption {
    label: String,
    target: ChoiceTarget,
}

#[derive(Clone)]
enum ChoiceTarget {
    Return,
    Node(String),
}

#[derive(Clone)]
enum DialogueStep {
    Line(DialogueLine),
    Choice(Vec<ChoiceOption>),
}

#[derive(Clone)]
struct LineEvent {
    trigger_index: usize,
    action: LineEventAction,
}

#[derive(Clone)]
enum LineEventAction {
    PlayAnim(String),
}

struct ParsedScript {
    is_female: bool,
    entries: Vec<DialogueStep>,
}

impl ParsedScript {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(path)?;
        Ok(parse_script_contents(&contents))
    }
}

#[derive(Resource)]
struct LiveDialogueData {
    is_female: bool,
    entries: Vec<DialogueStep>,
    current_index: usize,
    pending_line: Option<DialogueLine>,
    current_line: Option<usize>,
    pending_choice: Option<Vec<ChoiceOption>>,
    waiting_for_choice: bool,
    choice_dirty: bool,
    highlight_dirty: bool,
    needs_text_update: bool,
    needs_gender_sync: bool,
    finished: bool,
    last_modified: Option<SystemTime>,
}

impl Default for LiveDialogueData {
    fn default() -> Self {
        let mut data = Self {
            is_female: false,
            entries: Vec::new(),
            current_index: 0,
            pending_line: Some(DialogueLine {
                text: DIALOGUE_PLACEHOLDER.to_string(),
                events: Vec::new(),
                line_number: 0,
            }),
            current_line: None,
            pending_choice: None,
            waiting_for_choice: false,
            choice_dirty: true,
            highlight_dirty: true,
            needs_text_update: true,
            needs_gender_sync: true,
            finished: false,
            last_modified: None,
        };
        let path = live_root_path().join(DEFAULT_FILE);
        if let Ok(parsed) = ParsedScript::from_path(&path) {
            let modified = fs::metadata(&path)
                .ok()
                .and_then(|meta| meta.modified().ok());
            data.apply_parsed(parsed, modified);
        } else {
            warn!("初始化对话数据失败：无法读取 {}", path.display());
        }
        data
    }
}

impl LiveDialogueData {
    fn apply_parsed(&mut self, parsed: ParsedScript, modified: Option<SystemTime>) {
        self.is_female = parsed.is_female;
        self.entries = parsed.entries;
        self.current_index = 0;
        self.pending_line = None;
        self.current_line = None;
        self.pending_choice = None;
        self.waiting_for_choice = false;
        self.choice_dirty = true;
        self.highlight_dirty = true;
        self.finished = false;
        self.needs_gender_sync = true;
        self.last_modified = modified;
        self.queue_next_entry();
    }

    fn queue_next_entry(&mut self) {
        while self.current_index < self.entries.len() {
            match &self.entries[self.current_index] {
                DialogueStep::Line(line) => {
                    self.current_index += 1;
                    self.pending_line = Some(line.clone());
                    self.current_line = (line.line_number > 0).then_some(line.line_number);
                    self.highlight_dirty = true;
                    self.pending_choice = None;
                    self.waiting_for_choice = false;
                    self.choice_dirty = true;
                    self.needs_text_update = true;
                    return;
                }
                DialogueStep::Choice(options) => {
                    self.current_index += 1;
                    self.pending_choice = Some(options.clone());
                    self.waiting_for_choice = true;
                    self.choice_dirty = true;
                    self.needs_text_update = false;
                    return;
                }
            }
        }
        self.pending_line = Some(DialogueLine {
            text: DIALOGUE_PLACEHOLDER.to_string(),
            events: Vec::new(),
            line_number: 0,
        });
        self.current_line = None;
        self.highlight_dirty = true;
        self.pending_choice = None;
        self.waiting_for_choice = false;
        self.choice_dirty = true;
        self.needs_text_update = true;
        self.finished = true;
    }

    fn advance_text(&mut self) -> bool {
        if self.waiting_for_choice {
            return false;
        }
        if self.current_index >= self.entries.len() {
            if self.finished {
                return false;
            }
            self.finished = true;
            self.pending_line = Some(DialogueLine {
                text: DIALOGUE_FINISHED_LINE.to_string(),
                events: Vec::new(),
                line_number: 0,
            });
            self.current_line = None;
            self.highlight_dirty = true;
            self.needs_text_update = true;
            return true;
        }
        self.queue_next_entry();
        true
    }

    fn highlight_line(&self) -> Option<usize> {
        self.current_line
    }

    fn take_highlight_dirty(&mut self) -> bool {
        std::mem::take(&mut self.highlight_dirty)
    }

    fn take_choice_dirty(&mut self) -> bool {
        std::mem::take(&mut self.choice_dirty)
    }

    fn pending_choices(&self) -> Option<&[ChoiceOption]> {
        self.pending_choice.as_deref()
    }

    fn is_waiting_for_choice(&self) -> bool {
        self.waiting_for_choice
    }

    fn select_choice(&mut self, index: usize) -> bool {
        if !self.waiting_for_choice {
            return false;
        }
        let Some(options) = self.pending_choice.as_ref() else {
            return false;
        };
        let Some(option) = options.get(index) else {
            return false;
        };
        match &option.target {
            ChoiceTarget::Return => {
                self.pending_choice = None;
                self.waiting_for_choice = false;
                self.choice_dirty = true;
                self.pending_line = Some(DialogueLine {
                    text: DIALOGUE_FINISHED_LINE.to_string(),
                    events: Vec::new(),
                    line_number: 0,
                });
                self.current_line = None;
                self.highlight_dirty = true;
                self.needs_text_update = true;
                self.finished = true;
                self.current_index = self.entries.len();
            }
            ChoiceTarget::Node(_target) => {
                self.pending_choice = None;
                self.waiting_for_choice = false;
                self.choice_dirty = true;
                self.queue_next_entry();
            }
        }
        true
    }
}

fn parse_script_contents(contents: &str) -> ParsedScript {
    let is_female = extract_is_female(contents);
    let entries = collect_entries(contents, is_female);
    ParsedScript { is_female, entries }
}

fn extract_is_female(contents: &str) -> bool {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub const isFemale")
            && let Some(value) = trimmed.split('=').nth(1)
        {
            return value.to_ascii_lowercase().contains("true");
        }
    }
    false
}

fn collect_entries(contents: &str, is_female: bool) -> Vec<DialogueStep> {
    let mut entries = Vec::new();
    let mut contexts: Vec<ConditionalContext> = Vec::new();
    let mut lines = contents.lines().enumerate().peekable();
    while let Some((line_number, line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("if (isFemale") {
            contexts.push(ConditionalContext::new(is_female));
            continue;
        }
        if trimmed.starts_with("if (!isFemale") {
            contexts.push(ConditionalContext::new(!is_female));
            continue;
        }
        if trimmed.starts_with("} else {") {
            if let Some(ctx) = contexts.last_mut() {
                ctx.value = !ctx.cond_result;
            }
            continue;
        }
        if trimmed == "}" {
            contexts.pop();
            continue;
        }
        if !contexts.iter().all(|ctx| ctx.value) {
            continue;
        }
        if let Some(text) = parse_text_line(trimmed) {
            entries.push(DialogueStep::Line(DialogueLine {
                text,
                events: Vec::new(),
                line_number: line_number + 1,
            }));
            continue;
        }
        if trimmed.starts_with("with events") {
            if let Some(last) = entries.last_mut()
                && matches!(last, DialogueStep::Line(_))
            {
                if let DialogueStep::Line(line) = last {
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
                entries.push(DialogueStep::Choice(options));
            }
        }
    }
    entries
}

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

struct ConditionalContext {
    cond_result: bool,
    value: bool,
}

impl ConditionalContext {
    fn new(cond_result: bool) -> Self {
        Self {
            cond_result,
            value: cond_result,
        }
    }
}
