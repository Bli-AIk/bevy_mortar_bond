//! Split terminal + gameplay mock example.
//!
//! The window is divided into two panes: a faux Unix terminal on the left and a
//! placeholder gameplay viewport on the right. Click the terminal to capture
//! keyboard focus, run `bevim live_example`, and then edit the Mortar script in
//! a tiny vim-inspired editor. The right-hand view is left as a TODO hook for
//! real gameplay rendering.

use bevy::{
    ecs::message::MessageReader,
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
    },
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use std::{fs, path::Path};

const LIVE_EXAMPLE_NAME: &str = "live_example.mortar";
const LIVE_EXAMPLE_TEXT: &str = include_str!("../assets/live_example.mortar");
const OUTPUT_DIR: &str = "tmp_dir";

fn main() {
    App::new()
        .init_resource::<TerminalMachine>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Live Mortar Editor".into(),
                resolution: WindowResolution::new(1200, 720),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_ui)
        .add_systems(
            Update,
            (
                handle_panel_focus,
                handle_keyboard_controls,
                refresh_terminal_display,
                update_focus_visuals,
            ),
        )
        .run();
}

#[derive(Component)]
struct TerminalPanel;

#[derive(Component)]
struct GamePanel;

#[derive(Component)]
struct TerminalDisplay;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let font = asset_server.load("Unifont.otf");

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
                        Text::new("booting terminal"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.9, 0.8)),
                        TerminalDisplay,
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
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                ))
                .with_children(|game_panel| {
                    game_panel.spawn((
                        Text::new(
                            "Gameplay viewport placeholder\n\
                             TODO: swap this area with actual Bevy world rendering.",
                        ),
                        TextFont {
                            font,
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.85, 0.7)),
                    ));
                });
        });
}

fn handle_panel_focus(
    mut terminal_query: Query<&Interaction, (Changed<Interaction>, With<TerminalPanel>)>,
    mut game_query: Query<&Interaction, (Changed<Interaction>, With<GamePanel>)>,
    mut machine: ResMut<TerminalMachine>,
) {
    for interaction in &mut terminal_query {
        if *interaction == Interaction::Pressed {
            machine.set_focus(true);
        }
    }
    for interaction in &mut game_query {
        if *interaction == Interaction::Pressed {
            machine.set_focus(false);
        }
    }
}

fn handle_keyboard_controls(
    mut inputs: MessageReader<KeyboardInput>,
    mut machine: ResMut<TerminalMachine>,
) {
    if !machine.focused {
        return;
    }

    for input in inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
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
            KeyCode::ArrowLeft => machine.move_cursor_left(),
            KeyCode::ArrowRight => machine.move_cursor_right(),
            KeyCode::ArrowUp => machine.move_cursor_up(),
            KeyCode::ArrowDown => machine.move_cursor_down(),
            _ => {}
        }
    }
}

fn refresh_terminal_display(
    mut machine: ResMut<TerminalMachine>,
    mut texts: Query<&mut Text, With<TerminalDisplay>>,
) {
    if !machine.dirty {
        return;
    }

    let rendered = machine.render();
    for mut text in &mut texts {
        text.0 = rendered.clone();
    }
    machine.dirty = false;
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
                if trimmed == "bevim live_example" {
                    self.shell
                        .push_history("Opening live_example.mortar inside bevim...");
                    self.view = TerminalView::Vim(VimEditorState::from_source(
                        LIVE_EXAMPLE_NAME,
                        LIVE_EXAMPLE_TEXT,
                    ));
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
        if let TerminalView::Vim(editor) = &mut self.view {
            if !matches!(editor.mode, VimMode::Normal) {
                editor.enter_normal_mode("Exited to NORMAL mode");
                self.dirty = true;
            }
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
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_up();
            self.dirty = true;
        }
    }

    fn move_cursor_down(&mut self) {
        if let TerminalView::Vim(editor) = &mut self.view {
            editor.move_down();
            self.dirty = true;
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

    fn render(&self) -> String {
        match &self.view {
            TerminalView::Shell => self.shell.render(self.focused),
            TerminalView::Vim(editor) => editor.render(),
        }
    }
}

fn save_buffer(editor: &VimEditorState) -> std::io::Result<std::path::PathBuf> {
    let out_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(out_dir)?;
    let path = out_dir.join(&editor.file_name);
    fs::write(&path, editor.to_string())?;
    Ok(path)
}

#[derive(Default)]
struct ShellState {
    lines: Vec<String>,
    current_input: String,
}

impl ShellState {
    fn new() -> Self {
        let mut state = Self::default();
        state.lines.push("souprune dev shell".into());
        state
            .lines
            .push("Click the terminal to capture keyboard input.".into());
        state
            .lines
            .push("Type `bevim live_example` to edit the Mortar script.".into());
        state.lines.push("Use `clear` to reset the prompt.".into());
        state
    }

    fn render(&self, focused: bool) -> String {
        let mut output = self.lines.join("\n");
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str("> ");
        output.push_str(&self.current_input);
        if focused {
            output.push('|');
        } else {
            output.push_str("  [click left pane to focus]");
        }
        output
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
        self.current_input.clear();
        command
    }

    fn push_history(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    fn clear_history(&mut self) {
        self.lines.clear();
        *self = ShellState::new();
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
    file_name: String,
    buffer: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    mode: VimMode,
    command_buffer: String,
    status: String,
}

impl VimEditorState {
    fn from_source(name: &str, contents: &str) -> Self {
        let lines: Vec<Vec<char>> = if contents.is_empty() {
            vec![Vec::new()]
        } else {
            contents
                .lines()
                .map(|line| line.chars().collect())
                .collect()
        };

        Self {
            file_name: name.into(),
            buffer: lines,
            cursor_row: 0,
            cursor_col: 0,
            mode: VimMode::Normal,
            command_buffer: String::new(),
            status: "NORMAL mode — press i to edit, :wq to save".into(),
        }
    }

    fn render(&self) -> String {
        let mut output = format!("-- bevim: {} --\n", self.file_name);
        for (idx, line) in self.buffer.iter().enumerate() {
            output.push_str(&format!("{:>4} ", idx + 1));
            if idx == self.cursor_row {
                output.push_str(&self.with_cursor_marker(line));
            } else {
                for ch in line {
                    output.push(*ch);
                }
            }
            output.push('\n');
        }
        output.push_str(&format!("-- {} -- {}\n", self.mode.label(), self.status));
        if matches!(self.mode, VimMode::Command) {
            output.push(':');
            output.push_str(&self.command_buffer);
        } else if matches!(self.mode, VimMode::Insert) {
            output.push_str("-- INSERT -- Esc to leave");
        } else {
            output.push_str("Commands: i=insert, :w=save, :wq=save+quit, :q=quit");
        }
        output
    }

    fn with_cursor_marker(&self, line: &[char]) -> String {
        let mut rendered = String::new();
        for (idx, ch) in line.iter().enumerate() {
            if idx == self.cursor_col {
                rendered.push('|');
            }
            rendered.push(*ch);
        }
        if self.cursor_col == line.len() {
            rendered.push('|');
        }
        rendered
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
            let split_off = line.split_off(self.cursor_col);
            split_off
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

impl ToString for VimEditorState {
    fn to_string(&self) -> String {
        self.buffer
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
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
