//! Split terminal + gameplay mock example.
//!
//! The window is divided into two panes: a faux Unix terminal on the left and a
//! placeholder gameplay viewport on the right. Click the terminal to capture
//! keyboard focus, run `bevim live_example`, and then edit the Mortar script in
//! a tiny vim-inspired editor. The right-hand view is left as a TODO hook for
//! real gameplay rendering.

#[path = "utils/rogue_sprite.rs"]
mod rogue_sprite;

use bevy::{
    asset::AssetPlugin,
    ecs::message::MessageReader,
    input::{
        ButtonInput, ButtonState,
        keyboard::{KeyCode, KeyboardInput},
    },
    prelude::*,
    ui::widget::NodeImageMode,
    window::{PresentMode, WindowResolution},
};
use rogue_sprite::{
    RogueAnimation, RogueAnimationState, RogueGender, RogueSprite, RogueSpritePlugin,
    RogueSpritesheet,
};
use std::fmt::Display;
use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    time::Duration,
};

const DEFAULT_FILE: &str = "live_example.mortar";
const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const LIVE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/live");
const FONT_PATH: &str = "font/Unifont.otf";
const SHELL_COMMANDS: [&str; 2] = ["bevim live_example", "clear"];

fn main() {
    App::new()
        .init_resource::<TerminalMachine>()
        .init_resource::<CursorBlink>()
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
        .add_systems(Startup, setup_ui)
        .add_systems(
            Update,
            (
                tick_cursor_blink,
                handle_panel_focus,
                handle_keyboard_controls,
                handle_gender_buttons,
                update_gender_button_colors,
                control_preview_animation,
                refresh_terminal_display,
                update_focus_visuals,
            ),
        )
        .add_plugins(RogueSpritePlugin)
        .run();
}

#[derive(Component)]
struct TerminalPanel;

#[derive(Component)]
struct GamePanel;

#[derive(Component)]
struct TerminalDisplay;

#[derive(Component, Clone)]
struct TerminalFont(Handle<Font>);

#[derive(Component)]
struct RoguePreviewImage;

#[derive(Component)]
struct GenderButton {
    gender: RogueGender,
}

fn setup_ui(
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
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                ))
                .with_children(|game_panel| {
                    game_panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|setup| {
                            setup.spawn((
                                Text::new("SetUp"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 1.0)),
                            ));

                            setup
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    for (gender, label) in [
                                        (RogueGender::Male, "Male (rows 1-5)"),
                                        (RogueGender::Female, "Female (rows 6-10)"),
                                    ] {
                                        row.spawn((
                                            Button,
                                            GenderButton { gender },
                                            Node {
                                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                                border: UiRect::all(Val::Px(1.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.15, 0.18, 0.25)),
                                            BorderColor::all(Color::srgb(0.4, 0.6, 0.9)),
                                        ))
                                        .with_children(
                                            |button| {
                                                button.spawn((
                                                    Text::new(label),
                                                    TextFont {
                                                        font: font.clone(),
                                                        font_size: 18.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::srgb(0.9, 0.9, 1.0)),
                                                ));
                                            },
                                        );
                                    }
                                });
                        });

                    game_panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            align_items: AlignItems::Stretch,
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|preview_area| {
                            preview_area
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.06, 0.06, 0.09)),
                                    BorderColor::all(Color::srgb(0.3, 0.3, 0.4)),
                                ))
                                .with_children(|preview_box| {
                                    let sprite =
                                        RogueSprite::new(RogueGender::Male, RogueAnimation::Idle);
                                    let mut image = rogue_sheet.image_node(&sprite);
                                    image.image_mode = NodeImageMode::Stretch;
                                    preview_box.spawn((
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

                    game_panel.spawn((
                        Text::new(
                            "The character plays Idle by default.\n\
                             Press and hold the P key to preview the Walk animation, release to resume.\n\
                             Click this panel to capture this input.",
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

fn handle_panel_focus(
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

fn handle_gender_buttons(
    mut buttons: Query<(&Interaction, &GenderButton), (Changed<Interaction>, With<Button>)>,
    mut preview: Query<&mut RogueSprite, With<RoguePreviewImage>>,
) {
    let Ok(mut sprite) = preview.single_mut() else {
        return;
    };
    for (interaction, gender_button) in &mut buttons {
        if *interaction == Interaction::Pressed && sprite.gender != gender_button.gender {
            sprite.gender = gender_button.gender;
        }
    }
}

fn update_gender_button_colors(
    preview: Query<&RogueSprite, With<RoguePreviewImage>>,
    mut buttons: Query<(&GenderButton, &mut BackgroundColor)>,
) {
    let Ok(sprite) = preview.single() else {
        return;
    };
    for (button, mut color) in &mut buttons {
        let selected = button.gender == sprite.gender;
        color.0 = if selected {
            Color::srgb(0.25, 0.32, 0.55)
        } else {
            Color::srgb(0.15, 0.18, 0.25)
        };
    }
}

fn control_preview_animation(
    keys: Res<ButtonInput<KeyCode>>,
    mut preview: Query<&mut RogueSprite, With<RoguePreviewImage>>,
) {
    let Ok(mut sprite) = preview.single_mut() else {
        return;
    };
    let target = if keys.pressed(KeyCode::KeyP) {
        RogueAnimation::Walk
    } else {
        RogueAnimation::Idle
    };
    if sprite.animation != target {
        sprite.animation = target;
    }
}

fn handle_keyboard_controls(
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

fn refresh_terminal_display(
    mut commands: Commands,
    mut machine: ResMut<TerminalMachine>,
    display: Query<(Entity, &TerminalFont), With<TerminalDisplay>>,
    children: Query<&Children>,
    cursor: Res<CursorBlink>,
) {
    if !machine.dirty {
        return;
    }

    let cursor_visible = machine.focused && cursor.visible;
    let render = machine.render(cursor_visible);
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

fn despawn_recursive(entity: Entity, commands: &mut Commands, children: &Query<&Children>) {
    if let Ok(child_entities) = children.get(entity) {
        for child in child_entities.iter() {
            despawn_recursive(child, commands, children);
        }
    }
    commands.entity(entity).despawn();
}

fn tick_cursor_blink(
    time: Res<Time>,
    mut blink: ResMut<CursorBlink>,
    mut machine: ResMut<TerminalMachine>,
) {
    if blink.tick(time.delta()) && machine.focused {
        machine.dirty = true;
    }
}

fn update_focus_visuals(
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

#[derive(Resource)]
struct TerminalMachine {
    focused: bool,
    view: TerminalView,
    shell: ShellState,
    dirty: bool,
}

#[derive(Resource)]
struct CursorBlink {
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
    fn tick(&mut self, delta: Duration) -> bool {
        if self.timer.tick(delta).just_finished() {
            self.visible = !self.visible;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.timer.reset();
        self.visible = true;
    }
}

fn cursor_highlight_color() -> Color {
    Color::srgb(0.25, 0.4, 0.9)
}

#[derive(Default)]
struct TerminalRender {
    lines: Vec<StyledLine>,
}

impl TerminalRender {
    fn push_plain_line(&mut self, text: impl Into<String>, color: Color) {
        let mut line = StyledLine::default();
        line.push_segment(text.into(), color);
        self.lines.push(line);
    }

    fn push_line(&mut self, line: StyledLine) {
        self.lines.push(line);
    }
}

#[derive(Default)]
struct StyledLine {
    segments: Vec<StyledSegment>,
}

impl StyledLine {
    fn push_segment(&mut self, text: impl Into<String>, color: Color) {
        self.push_segment_with_bg(text, color, None);
    }

    fn push_segment_with_bg(
        &mut self,
        text: impl Into<String>,
        color: Color,
        background: Option<Color>,
    ) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        if background.is_none()
            && let Some(last) = self.segments.last_mut()
            && last.color == color
            && last.background.is_none()
        {
            last.text.push_str(&text);
            return;
        }

        self.segments.push(StyledSegment {
            text,
            color,
            background,
        });
    }
}

struct StyledSegment {
    text: String,
    color: Color,
    background: Option<Color>,
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
    fn set_focus(&mut self, focused: bool) {
        if self.focused != focused {
            self.focused = focused;
            self.dirty = true;
        }
    }

    fn handle_text_character(&mut self, ch: char) {
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

    fn handle_enter(&mut self) {
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

    fn handle_backspace(&mut self) {
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

    fn handle_escape(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view
            && !matches!(editor.mode, VimMode::Normal)
        {
            editor.enter_normal_mode("Exited to NORMAL mode");
            self.dirty = true;
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

    fn handle_tab(&mut self) {
        if let TerminalView::Shell = self.view
            && self.shell.autocomplete(&SHELL_COMMANDS)
        {
            self.dirty = true;
        }
    }

    fn move_cursor_left(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_left();
            self.dirty = true;
        }
    }

    fn move_cursor_right(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_right();
            self.dirty = true;
        }
    }

    fn move_cursor_up(&mut self) {
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

    fn move_cursor_down(&mut self) {
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

    fn render(&self, cursor_visible: bool) -> TerminalRender {
        match &self.view {
            TerminalView::Shell => self.shell.render(self.focused, cursor_visible),
            TerminalView::Vim(editor) => editor.render(cursor_visible),
        }
    }
}

fn save_buffer(editor: &VimEditorState) -> std::io::Result<PathBuf> {
    let path = live_root_path().join(&editor.relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, editor.to_string())?;
    Ok(path)
}

fn build_editor_for_target(target: &str) -> Result<VimEditorState, String> {
    let relative = sanitize_live_target(target)?;
    let path = live_root_path().join(&relative);
    let highlight = path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("mortar"))
        .unwrap_or(false);
    let contents = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err.to_string()),
    };
    Ok(VimEditorState::from_source(relative, contents, highlight))
}

fn live_root_path() -> &'static Path {
    Path::new(LIVE_ROOT)
}

fn sanitize_live_target(target: &str) -> Result<PathBuf, String> {
    let trimmed = target.trim();
    let fallback = if trimmed.is_empty() {
        DEFAULT_FILE
    } else {
        trimmed
    };
    let mut relative = PathBuf::new();
    for component in Path::new(fallback).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => continue,
            _ => return Err("path must stay within assets/live/".into()),
        }
    }
    if relative.as_os_str().is_empty() {
        relative.push(DEFAULT_FILE);
    }
    if relative.extension().is_none() {
        relative.set_extension("mortar");
    }
    Ok(relative)
}

#[derive(Default)]
struct ShellState {
    lines: Vec<String>,
    current_input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    saved_input: Option<String>,
}

impl ShellState {
    fn new() -> Self {
        let mut state = Self::default();
        state.reset_lines();
        state
    }

    fn reset_lines(&mut self) {
        self.lines.push("souprune dev shell".into());
        self.lines
            .push("Click the terminal to capture keyboard input.".into());
        self.lines
            .push("Type `bevim live_example` to edit files under assets/live/.".into());
        self.lines.push("Use `clear` to reset the prompt.".into());
        self.lines
            .push("Paths stay inside assets/live/ and default to `.mortar`.".into());
    }

    fn render(&self, focused: bool, cursor_visible: bool) -> TerminalRender {
        let mut render = TerminalRender::default();
        for line in &self.lines {
            render.push_plain_line(line, Color::srgb(0.8, 0.9, 0.8));
        }
        let mut prompt = StyledLine::default();
        prompt.push_segment("> ", Color::srgb(0.6, 0.9, 0.7));
        prompt.push_segment(&self.current_input, Color::srgb(0.9, 0.9, 0.9));
        if focused {
            if cursor_visible {
                prompt.push_segment_with_bg(
                    " ".to_string(),
                    Color::srgb(0.9, 0.9, 0.9),
                    Some(cursor_highlight_color()),
                );
            }
        } else {
            prompt.push_segment("  [click left pane to focus]", Color::srgb(0.5, 0.7, 1.0));
        }
        render.push_line(prompt);
        render
    }

    fn push_char(&mut self, ch: char) {
        self.current_input.push(ch);
    }

    fn backspace(&mut self) {
        self.current_input.pop();
    }

    fn finish_command(&mut self) -> String {
        let command = self.current_input.clone();
        if !command.is_empty() {
            self.lines.push(format!("> {}", command));
        }
        if !command.trim().is_empty() {
            self.history.push(command.clone());
            self.history_index = None;
            self.saved_input = None;
        }
        self.current_input.clear();
        command
    }

    fn push_history(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    fn clear_history(&mut self) {
        self.lines.clear();
        self.reset_lines();
        self.current_input.clear();
        self.history_index = None;
        self.saved_input = None;
    }

    fn history_previous(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        let next_index = match self.history_index {
            None => {
                self.saved_input = Some(self.current_input.clone());
                Some(self.history.len().saturating_sub(1))
            }
            Some(idx) if idx > 0 => Some(idx - 1),
            Some(idx) => Some(idx),
        };
        if let Some(idx) = next_index {
            self.history_index = Some(idx);
            self.current_input = self.history[idx].clone();
            true
        } else {
            false
        }
    }

    fn history_next(&mut self) -> bool {
        let Some(idx) = self.history_index else {
            return false;
        };
        if idx + 1 < self.history.len() {
            let new_index = idx + 1;
            self.history_index = Some(new_index);
            self.current_input = self.history[new_index].clone();
            true
        } else {
            self.history_index = None;
            if let Some(saved) = self.saved_input.take() {
                self.current_input = saved;
            } else {
                self.current_input.clear();
            }
            true
        }
    }

    fn autocomplete(&mut self, commands: &[&str]) -> bool {
        if self.current_input.trim().is_empty() {
            if let Some(first) = commands.first() {
                self.current_input = (*first).to_string();
                self.history_index = None;
                self.saved_input = None;
                return true;
            }
            return false;
        }
        if let Some(matched) = commands
            .iter()
            .find(|candidate| candidate.starts_with(&self.current_input))
        {
            self.current_input = (**matched).to_string();
            self.history_index = None;
            self.saved_input = None;
            return true;
        }
        false
    }
}

enum TerminalView {
    Shell,
    Vim(VimEditorState),
}

impl TerminalView {
    fn as_vim_mut(&mut self) -> Option<&mut VimEditorState> {
        match self {
            TerminalView::Vim(editor) => Some(editor),
            TerminalView::Shell => None,
        }
    }
}

struct VimEditorState {
    relative_path: PathBuf,
    display_name: String,
    highlight: bool,
    buffer: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    mode: VimMode,
    command_buffer: String,
    status: String,
}

impl VimEditorState {
    fn from_source(relative_path: PathBuf, contents: String, highlight: bool) -> Self {
        let lines: Vec<Vec<char>> = if contents.is_empty() {
            vec![Vec::new()]
        } else {
            contents
                .lines()
                .map(|line| line.chars().collect())
                .collect()
        };

        let display_name = relative_path.to_string_lossy().into_owned();

        Self {
            relative_path,
            display_name,
            highlight,
            buffer: lines,
            cursor_row: 0,
            cursor_col: 0,
            mode: VimMode::Normal,
            command_buffer: String::new(),
            status: "NORMAL mode — press i to edit, :wq to save".into(),
        }
    }

    fn render(&self, cursor_visible: bool) -> TerminalRender {
        let mut render = TerminalRender::default();
        render.push_plain_line(
            format!("-- bevim: {} --", self.display_name),
            Color::srgb(0.9, 0.85, 0.5),
        );

        for (idx, _) in self.buffer.iter().enumerate() {
            let mut line = StyledLine::default();
            line.push_segment(format!("{:>4} ", idx + 1), Color::srgb(0.6, 0.6, 0.9));
            let segments_line = if self.highlight {
                self.highlight_line(idx, cursor_visible)
            } else {
                self.plain_line(idx, cursor_visible)
            };
            for segment in segments_line.segments {
                line.push_segment_with_bg(segment.text, segment.color, segment.background);
            }
            render.push_line(line);
        }

        let mut status_line = StyledLine::default();
        status_line.push_segment(
            format!("-- {} -- {}", self.mode.label(), self.status),
            Color::srgb(0.8, 0.8, 0.5),
        );
        render.push_line(status_line);

        match self.mode {
            VimMode::Command => {
                let mut command_line = StyledLine::default();
                command_line.push_segment(":", Color::srgb(0.7, 0.9, 1.0));
                command_line.push_segment(&self.command_buffer, Color::srgb(0.9, 0.9, 0.9));
                if cursor_visible {
                    command_line.push_segment_with_bg(
                        " ".to_string(),
                        Color::srgb(0.9, 0.9, 0.9),
                        Some(cursor_highlight_color()),
                    );
                }
                render.push_line(command_line);
            }
            VimMode::Insert => {
                render.push_plain_line("-- INSERT -- Esc to leave", Color::srgb(0.7, 1.0, 0.8));
            }
            VimMode::Normal => {
                render.push_plain_line(
                    "Commands: i=insert, :w=save, :wq=save+quit, :q=quit",
                    Color::srgb(0.6, 0.8, 1.0),
                );
            }
        }

        render
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn plain_line(&self, row: usize, cursor_visible: bool) -> StyledLine {
        let mut line = StyledLine::default();
        let Some(chars) = self.buffer.get(row) else {
            return line;
        };
        let cursor_col = if cursor_visible && row == self.cursor_row {
            Some(self.cursor_col.min(chars.len()))
        } else {
            None
        };
        self.push_token_with_cursor(
            &mut line,
            chars,
            0,
            chars.len(),
            Color::srgb(0.9, 0.9, 0.9),
            cursor_col,
            cursor_highlight_color(),
        );
        if matches!(cursor_col, Some(col) if col == chars.len()) {
            line.push_segment_with_bg(
                " ".to_string(),
                Color::srgb(0.9, 0.9, 0.9),
                Some(cursor_highlight_color()),
            );
        }
        line
    }

    fn highlight_line(&self, row: usize, cursor_visible: bool) -> StyledLine {
        let mut line = StyledLine::default();
        let Some(chars) = self.buffer.get(row) else {
            return line;
        };
        let cursor_col = if cursor_visible && row == self.cursor_row {
            Some(self.cursor_col.min(chars.len()))
        } else {
            None
        };
        let cursor_bg = cursor_highlight_color();

        let mut idx = 0;
        while idx < chars.len() {
            let ch = chars[idx];
            if ch == '/' && idx + 1 < chars.len() && chars[idx + 1] == '/' {
                self.push_token_with_cursor(
                    &mut line,
                    chars,
                    idx,
                    chars.len(),
                    Color::srgb(0.6, 0.8, 0.6),
                    cursor_col,
                    cursor_bg,
                );
                return line;
            }

            if ch == '"' {
                let start = idx;
                idx += 1;
                while idx < chars.len() {
                    let c = chars[idx];
                    idx += 1;
                    if c == '"' {
                        break;
                    }
                }
                self.push_token_with_cursor(
                    &mut line,
                    chars,
                    start,
                    idx,
                    Color::srgb(0.7, 1.0, 0.7),
                    cursor_col,
                    cursor_bg,
                );
                continue;
            }

            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = idx;
                while idx < chars.len() && (chars[idx].is_alphanumeric() || chars[idx] == '_') {
                    idx += 1;
                }
                let word_color = match chars[start..idx].iter().collect::<String>().as_str() {
                    "node" | "text" | "choice" | "return" => Color::srgb(0.6, 0.8, 1.0),
                    _ => Color::srgb(0.9, 0.9, 0.9),
                };
                self.push_token_with_cursor(
                    &mut line, chars, start, idx, word_color, cursor_col, cursor_bg,
                );
                continue;
            }

            if ch == '-' && idx + 1 < chars.len() && chars[idx + 1] == '>' {
                self.push_token_with_cursor(
                    &mut line,
                    chars,
                    idx,
                    idx + 2,
                    Color::srgb(0.9, 0.7, 0.5),
                    cursor_col,
                    cursor_bg,
                );
                idx += 2;
                continue;
            }

            let color = match ch {
                '{' | '}' | '[' | ']' => Color::srgb(1.0, 0.7, 0.5),
                ':' | ',' => Color::srgb(0.8, 0.6, 0.9),
                _ => Color::srgb(0.9, 0.9, 0.9),
            };
            self.push_token_with_cursor(
                &mut line,
                chars,
                idx,
                idx + 1,
                color,
                cursor_col,
                cursor_bg,
            );
            idx += 1;
        }

        if matches!(cursor_col, Some(col) if col == chars.len()) {
            line.push_segment_with_bg(" ", Color::srgb(0.9, 0.9, 0.9), Some(cursor_bg));
        }

        line
    }

    fn push_token_with_cursor(
        &self,
        line: &mut StyledLine,
        chars: &[char],
        start: usize,
        end: usize,
        color: Color,
        cursor_col: Option<usize>,
        cursor_bg: Color,
    ) {
        if start >= end {
            return;
        }
        if let Some(cursor) = cursor_col
            && cursor >= start
            && cursor < end
        {
            if cursor > start {
                line.push_segment(chars[start..cursor].iter().collect::<String>(), color);
            }
            let mut cursor_char = String::new();
            cursor_char.push(chars[cursor]);
            line.push_segment_with_bg(cursor_char, color, Some(cursor_bg));
            if cursor + 1 < end {
                line.push_segment(chars[cursor + 1..end].iter().collect::<String>(), color);
            }
            return;
        }
        line.push_segment(chars[start..end].iter().collect::<String>(), color);
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(line) = self.buffer.get_mut(self.cursor_row) {
            line.insert(self.cursor_col, ch);
            self.cursor_col += 1;
        }
    }

    fn insert_newline(&mut self) {
        if self.cursor_row >= self.buffer.len() {
            self.buffer.push(Vec::new());
            self.cursor_row = self.buffer.len() - 1;
            self.cursor_col = 0;
            return;
        }
        let tail = {
            let line = &mut self.buffer[self.cursor_row];
            line.split_off(self.cursor_col)
        };
        self.cursor_row += 1;
        self.buffer.insert(self.cursor_row, tail);
        self.cursor_col = 0;
    }

    fn backspace(&mut self) {
        if self.cursor_row >= self.buffer.len() {
            return;
        }
        if self.cursor_col > 0 {
            if let Some(line) = self.buffer.get_mut(self.cursor_row) {
                self.cursor_col -= 1;
                if self.cursor_col < line.len() {
                    line.remove(self.cursor_col);
                }
            }
        } else if self.cursor_row > 0 {
            let tail = self.buffer.remove(self.cursor_row);
            self.cursor_row -= 1;
            let prev_len = self.buffer[self.cursor_row].len();
            self.buffer[self.cursor_row].extend(tail);
            self.cursor_col = prev_len;
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.buffer[self.cursor_row].len();
        }
    }

    fn move_right(&mut self) {
        if self.cursor_row >= self.buffer.len() {
            return;
        }
        let len = self.buffer[self.cursor_row].len();
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.buffer.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.buffer.len() {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    fn clamp_col(&mut self) {
        if let Some(line) = self.buffer.get(self.cursor_row) {
            self.cursor_col = self.cursor_col.min(line.len());
        }
    }

    fn enter_insert_mode(&mut self) {
        self.mode = VimMode::Insert;
        self.status = "INSERT mode — Esc to return to NORMAL".into();
    }

    fn enter_command_mode(&mut self) {
        self.mode = VimMode::Command;
        self.command_buffer.clear();
        self.status = "COMMAND mode — type :w, :wq, or :q".into();
    }

    fn enter_normal_mode(&mut self, message: &str) {
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
        self.status = message.into();
    }

    fn push_command_char(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    fn submit_command(&mut self) -> VimCommandAction {
        let command = self.command_buffer.trim().to_string();
        self.command_buffer.clear();
        self.mode = VimMode::Normal;
        match command.as_str() {
            "w" => VimCommandAction::Save { quit: false },
            "wq" | "x" => VimCommandAction::Save { quit: true },
            "q" => VimCommandAction::Quit,
            "" => {
                self.status = "Command aborted".into();
                VimCommandAction::None
            }
            other => {
                self.status = format!("E492: Not an editor command: {}", other);
                VimCommandAction::None
            }
        }
    }

    fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

impl Display for VimEditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .buffer
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", str)
    }
}

enum VimMode {
    Normal,
    Insert,
    Command,
}

impl VimMode {
    fn label(&self) -> &'static str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Command => "COMMAND",
        }
    }
}

enum VimCommandAction {
    None,
    Save { quit: bool },
    Quit,
}
