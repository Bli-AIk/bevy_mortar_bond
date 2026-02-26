#[path = "live_terminal_editor.rs"]
mod live_terminal_editor;
pub use live_terminal_editor::live_root_path;
use live_terminal_editor::*;

use super::{
    rogue_sprite::{
        RogueAnimation, RogueAnimationState, RogueGender, RogueSprite, RogueSpritesheet,
    },
    typewriter::{Typewriter, TypewriterState},
};
use bevy::{
    ecs::message::MessageReader,
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
    },
    prelude::*,
    ui::widget::NodeImageMode,
};
use bevy_mortar_bond::MortarRuntime;
use std::time::Duration;
pub const DEFAULT_FILE: &str = "live_example.mortar";
pub const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
pub const LIVE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/live");
pub const FONT_PATH: &str = "font/Unifont.otf";
pub const SHELL_COMMANDS: [&str; 2] = ["bevim live_example", "clear"];
pub const DIALOGUE_CHAR_SPEED: f32 = 0.04;
pub const DIALOGUE_PLACEHOLDER: &str = "Start editing live_example.mortar to drive the dialogue.";

#[derive(Component)]
pub struct TerminalPanel;

#[derive(Component)]
pub struct GamePanel;

#[derive(Component)]
pub struct TerminalDisplay;

#[derive(Component, Clone)]
pub struct TerminalFont(pub Handle<Font>);

#[derive(Component)]
pub struct RoguePreviewImage;

#[derive(Component)]
pub struct GameDialogueText;

#[derive(Component)]
pub struct ChoicePanel;

#[derive(Component)]
pub struct ChoiceButton {
    pub index: usize,
}

#[derive(Component, Clone)]
pub struct ChoicePanelFont(pub Handle<Font>);

#[derive(Component)]
pub struct AnimationRevertTimer(pub Timer);

pub trait ScriptHighlightSource {
    fn highlight_line(&self, runtime: &MortarRuntime) -> Option<usize>;
}

#[derive(Resource)]
pub struct TerminalMachine {
    pub focused: bool,
    pub view: TerminalView,
    pub shell: ShellState,
    pub dirty: bool,
}

impl Default for TerminalMachine {
    fn default() -> Self {
        Self {
            focused: false,
            view: TerminalView::Shell,
            shell: ShellState::new(),
            dirty: true,
        }
    }
}

impl TerminalMachine {
    pub fn set_focus(&mut self, focused: bool) {
        if self.focused != focused {
            self.focused = focused;
            self.dirty = true;
        }
    }

    pub fn handle_text_character(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        match &mut self.view {
            TerminalView::Shell => {
                self.shell.push_char(ch);
                self.dirty = true;
            }
            TerminalView::Vim(editor) => match editor.mode {
                VimMode::Insert => {
                    editor.insert_char(ch);
                    self.dirty = true;
                }
                VimMode::Normal => {
                    let handled = match ch {
                        'h' => {
                            editor.move_left();
                            true
                        }
                        'j' => {
                            editor.move_down();
                            true
                        }
                        'k' => {
                            editor.move_up();
                            true
                        }
                        'l' => {
                            editor.move_right();
                            true
                        }
                        'i' => {
                            editor.enter_insert_mode();
                            true
                        }
                        ':' => {
                            editor.enter_command_mode();
                            true
                        }
                        _ => false,
                    };
                    if handled {
                        self.dirty = true;
                    }
                }
                VimMode::Command => {
                    editor.push_command_char(ch);
                    self.dirty = true;
                }
            },
        }
    }

    pub fn handle_enter(&mut self) {
        match &mut self.view {
            TerminalView::Shell => {
                let command = self.shell.finish_command();
                let trimmed = command.trim();
                if trimmed.is_empty() {
                    return;
                }
                if let Some(rest) = trimmed.strip_prefix("bevim") {
                    self.launch_bevim(rest.trim());
                } else if trimmed == "clear" {
                    self.shell.clear_history();
                } else {
                    self.shell
                        .push_history(format!("command not found: {}", trimmed));
                }
                self.dirty = true;
            }
            TerminalView::Vim(editor) => match editor.mode {
                VimMode::Insert => {
                    editor.insert_newline();
                    self.dirty = true;
                }
                VimMode::Command => {
                    let action = editor.submit_command();
                    self.apply_vim_command(action);
                }
                VimMode::Normal => {}
            },
        }
    }

    pub fn handle_backspace(&mut self) {
        match &mut self.view {
            TerminalView::Shell => {
                self.shell.backspace();
                self.dirty = true;
            }
            TerminalView::Vim(editor) => match editor.mode {
                VimMode::Insert => {
                    editor.backspace();
                    self.dirty = true;
                }
                VimMode::Command => {
                    editor.command_backspace();
                    self.dirty = true;
                }
                VimMode::Normal => {}
            },
        }
    }

    pub fn handle_escape(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view
            && !matches!(editor.mode, VimMode::Normal)
        {
            editor.enter_normal_mode("Exited to NORMAL mode");
            self.dirty = true;
        }
    }

    pub fn handle_tab(&mut self) {
        if let TerminalView::Shell = self.view
            && self.shell.autocomplete(&SHELL_COMMANDS)
        {
            self.dirty = true;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_left();
            self.dirty = true;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_right();
            self.dirty = true;
        }
    }

    pub fn move_cursor_up(&mut self) {
        match &mut self.view {
            TerminalView::Vim(editor) => {
                editor.move_up();
                self.dirty = true;
            }
            TerminalView::Shell => {
                if self.shell.history_previous() {
                    self.dirty = true;
                }
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        match &mut self.view {
            TerminalView::Vim(editor) => {
                editor.move_down();
                self.dirty = true;
            }
            TerminalView::Shell => {
                if self.shell.history_next() {
                    self.dirty = true;
                }
            }
        }
    }

    pub fn render(&self, cursor_visible: bool, highlight_line: Option<usize>) -> TerminalRender {
        match &self.view {
            TerminalView::Shell => self.shell.render(self.focused, cursor_visible),
            TerminalView::Vim(editor) => editor.render(cursor_visible, highlight_line),
        }
    }

    fn launch_bevim(&mut self, target: &str) {
        let result = build_editor_for_target(target);
        match result {
            Ok(editor) => {
                let name = editor.display_name().to_string();
                self.shell
                    .push_history(format!("Opening {} inside bevim...", name));
                self.view = TerminalView::Vim(editor);
            }
            Err(err) => {
                self.shell.push_history(format!("bevim: {}", err));
            }
        }
        self.dirty = true;
    }

    fn apply_vim_command(&mut self, action: VimCommandAction) {
        match action {
            VimCommandAction::None => {
                self.dirty = true;
            }
            VimCommandAction::Save { quit } => {
                let Some(editor) = self.view.as_vim_mut() else {
                    return;
                };
                let result = save_buffer(editor);
                match result {
                    Ok(path) => {
                        editor.set_status(format!("written {}", path.display()));
                        if quit {
                            self.exit_vim_with_message("wrote changes and closed bevim");
                        }
                    }
                    Err(err) => {
                        editor.set_status(format!("write failed: {}", err));
                    }
                }
                self.dirty = true;
            }
            VimCommandAction::Quit => {
                self.exit_vim_with_message("Exited bevim without saving");
            }
        }
    }

    fn exit_vim_with_message(&mut self, message: &str) {
        self.shell.push_history(format!("[bevim] {}", message));
        self.view = TerminalView::Shell;
        self.dirty = true;
    }
}

#[derive(Resource)]
pub struct CursorBlink {
    timer: Timer,
    visible: bool,
}

impl Default for CursorBlink {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true,
        }
    }
}

impl CursorBlink {
    pub fn tick(&mut self, delta: Duration) -> bool {
        if self.timer.tick(delta).just_finished() {
            self.visible = !self.visible;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.timer.reset();
        self.visible = true;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}


pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rogue_sheet: Res<RogueSpritesheet>,
) {
    commands.spawn(Camera2d);
    let font = asset_server.load(FONT_PATH);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    TerminalPanel,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
                    BorderColor::all(Color::srgb(0.4, 0.7, 1.0)),
                ))
                .with_children(|terminal| {
                    terminal.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        TerminalDisplay,
                        TerminalFont(font.clone()),
                    ));
                });

            parent
                .spawn((
                    Button,
                    GamePanel,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                ))
                .with_children(|game_panel| {
                    game_panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|column| {
                            column
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(12.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                                    BorderColor::all(Color::srgb(0.4, 0.5, 0.7)),
                                ))
                                .with_children(|dialogue_box| {
                                    let mut initial_typewriter =
                                        Typewriter::new(DIALOGUE_PLACEHOLDER, DIALOGUE_CHAR_SPEED);
                                    initial_typewriter.current_text =
                                        DIALOGUE_PLACEHOLDER.to_string();
                                    initial_typewriter.current_char_index =
                                        DIALOGUE_PLACEHOLDER.chars().count();
                                    initial_typewriter.state = TypewriterState::Finished;
                                    dialogue_box.spawn((
                                        Text::new(DIALOGUE_PLACEHOLDER),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                                        GameDialogueText,
                                        initial_typewriter,
                                    ));
                                });

                            column.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(8.0),
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.06, 0.06, 0.09)),
                                BorderColor::all(Color::srgb(0.35, 0.35, 0.45)),
                                ChoicePanel,
                                Visibility::Hidden,
                                ChoicePanelFont(font.clone()),
                            ));

                            column
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_grow: 1.0,
                                        align_items: AlignItems::Stretch,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.06, 0.06, 0.09)),
                                    BorderColor::all(Color::srgb(0.3, 0.3, 0.4)),
                                ))
                                .with_children(|preview_box| {
                                    preview_box
                                        .spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(100.0),
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|center| {
                                            let sprite = RogueSprite::new(
                                                RogueGender::Male,
                                                RogueAnimation::Idle,
                                            );
                                            let mut image = rogue_sheet.image_node(&sprite);
                                            image.image_mode = NodeImageMode::Stretch;
                                            center.spawn((
                                                RoguePreviewImage,
                                                sprite,
                                                RogueAnimationState::default(),
                                                image,
                                                Node {
                                                    width: Val::Px(192.0),
                                                    height: Val::Px(192.0),
                                                    ..default()
                                                },
                                            ));
                                        });
                                });
                        });

                    game_panel.spawn((
                        Text::new(
                            "Press Z to advance dialogue; click the buttons when choices appear.\n\
                             The `isFemale` constant in `live_example.mortar` decides the character's gender.\n\
                             Click this panel to capture input.\n\
                             Sprite from https://opengameart.org/content/animated-rogue",
                        ),
                        TextFont {
                            font,
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.85, 1.0)),
                    ));
                });
        });
}

pub fn handle_panel_focus(
    mut terminal_query: Query<&Interaction, (Changed<Interaction>, With<TerminalPanel>)>,
    mut game_query: Query<&Interaction, (Changed<Interaction>, With<GamePanel>)>,
    mut machine: ResMut<TerminalMachine>,
    mut blink: ResMut<CursorBlink>,
) {
    for interaction in &mut terminal_query {
        if *interaction == Interaction::Pressed {
            machine.set_focus(true);
            blink.reset();
        }
    }
    for interaction in &mut game_query {
        if *interaction == Interaction::Pressed {
            machine.set_focus(false);
        }
    }
}

pub fn handle_keyboard_controls(
    mut inputs: MessageReader<KeyboardInput>,
    mut machine: ResMut<TerminalMachine>,
    mut blink: ResMut<CursorBlink>,
) {
    if !machine.focused {
        return;
    }

    for input in inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        blink.reset();
        if let Some(text) = &input.text {
            for ch in text.chars() {
                if !ch.is_control() {
                    machine.handle_text_character(ch);
                }
            }
        }
        match input.key_code {
            KeyCode::Enter => machine.handle_enter(),
            KeyCode::Backspace => machine.handle_backspace(),
            KeyCode::Escape => machine.handle_escape(),
            KeyCode::Tab => machine.handle_tab(),
            KeyCode::ArrowLeft => machine.move_cursor_left(),
            KeyCode::ArrowRight => machine.move_cursor_right(),
            KeyCode::ArrowUp => machine.move_cursor_up(),
            KeyCode::ArrowDown => machine.move_cursor_down(),
            _ => {}
        }
    }
}
pub fn refresh_terminal_display<S: ScriptHighlightSource + Resource>(
    mut commands: Commands,
    mut machine: ResMut<TerminalMachine>,
    display: Query<(Entity, &TerminalFont), With<TerminalDisplay>>,
    children: Query<&Children>,
    cursor: Res<CursorBlink>,
    source: Res<S>,
    runtime: Res<MortarRuntime>,
) {
    if !machine.dirty {
        return;
    }

    let cursor_visible = machine.focused && cursor.visible();
    let highlight_line = source.as_ref().highlight_line(&runtime);
    let render = machine.render(cursor_visible, highlight_line);
    if let Ok((entity, font)) = display.single() {
        if let Ok(existing_children) = children.get(entity) {
            for child in existing_children.iter() {
                despawn_recursive(child, &mut commands, &children);
            }
        }
        commands.entity(entity).with_children(|parent| {
            for line in render.lines {
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|line_parent| {
                        for segment in line.segments {
                            let mut child = line_parent.spawn((
                                Text::new(segment.text),
                                TextFont {
                                    font: font.0.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(segment.color),
                            ));
                            if let Some(bg) = segment.background {
                                child.insert(TextBackgroundColor(bg));
                            }
                        }
                    });
            }
        });
    }

    machine.dirty = false;
}

pub fn despawn_recursive(entity: Entity, commands: &mut Commands, children: &Query<&Children>) {
    if let Ok(child_entities) = children.get(entity) {
        for child in child_entities.iter() {
            despawn_recursive(child, commands, children);
        }
    }
    commands.entity(entity).despawn();
}

pub fn tick_cursor_blink(
    time: Res<Time>,
    mut blink: ResMut<CursorBlink>,
    mut machine: ResMut<TerminalMachine>,
) {
    if blink.tick(time.delta()) && machine.focused {
        machine.dirty = true;
    }
}

pub fn update_focus_visuals(
    machine: Res<TerminalMachine>,
    mut terminal_border: Query<&mut BorderColor, (With<TerminalPanel>, Without<GamePanel>)>,
    mut game_border: Query<&mut BorderColor, (With<GamePanel>, Without<TerminalPanel>)>,
) {
    for mut border in &mut terminal_border {
        let color = if machine.focused {
            Color::srgb(0.6, 0.9, 1.0)
        } else {
            Color::srgb(0.4, 0.7, 1.0)
        };
        *border = BorderColor::all(color);
    }
    for mut border in &mut game_border {
        let color = if machine.focused {
            Color::srgb(0.5, 0.5, 0.5)
        } else {
            Color::srgb(0.7, 0.7, 0.7)
        };
        *border = BorderColor::all(color);
    }
}

#[derive(Message)]
pub struct RogueAnimationEvent {
    pub animation: RogueAnimation,
}

pub fn apply_animation_events(
    mut commands: Commands,
    mut events: MessageReader<RogueAnimationEvent>,
    mut preview: Query<(Entity, &mut RogueSprite), With<RoguePreviewImage>>,
) {
    let Ok((entity, mut sprite)) = preview.single_mut() else {
        for _ in events.read() {}
        return;
    };
    for event in events.read() {
        if sprite.animation != event.animation {
            sprite.animation = event.animation;
        }
        if matches!(event.animation, RogueAnimation::Attack) {
            commands
                .entity(entity)
                .insert(AnimationRevertTimer(Timer::from_seconds(
                    0.8,
                    TimerMode::Once,
                )));
        }
    }
}

pub fn revert_animation_to_idle(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut AnimationRevertTimer, &mut RogueSprite),
        With<RoguePreviewImage>,
    >,
) {
    for (entity, mut timer, mut sprite) in &mut query {
        if timer.0.tick(time.delta()).is_finished() {
            sprite.animation = RogueAnimation::Idle;
            commands.entity(entity).remove::<AnimationRevertTimer>();
        }
    }
}

pub fn animation_from_label(label: &str) -> Option<RogueAnimation> {
    match label.to_ascii_lowercase().as_str() {
        "idle" => Some(RogueAnimation::Idle),
        "walk" => Some(RogueAnimation::Walk),
        "attack" => Some(RogueAnimation::Attack),
        "gesture" => Some(RogueAnimation::Gesture),
        "death" => Some(RogueAnimation::Death),
        _ => None,
    }
}


