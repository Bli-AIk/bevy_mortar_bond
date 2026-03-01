//! Vim editor, shell state, and terminal rendering primitives for the live terminal.

use bevy::prelude::*;
use std::{
    fmt::Display,
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

#[derive(Default)]
pub struct TerminalRender {
    pub lines: Vec<StyledLine>,
}

#[derive(Default)]
pub struct StyledLine {
    pub segments: Vec<StyledSegment>,
}

pub struct StyledSegment {
    pub text: String,
    pub color: Color,
    pub background: Option<Color>,
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

pub enum TerminalView {
    Shell,
    Vim(VimEditorState),
}

impl TerminalView {
    pub fn as_vim_mut(&mut self) -> Option<&mut VimEditorState> {
        match self {
            TerminalView::Vim(editor) => Some(editor),
            TerminalView::Shell => None,
        }
    }
}

pub struct VimEditorState {
    relative_path: PathBuf,
    display_name: String,
    highlight: bool,
    buffer: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    pub mode: VimMode,
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

    pub fn render(&self, cursor_visible: bool, highlight_line: Option<usize>) -> TerminalRender {
        let mut render = TerminalRender::default();
        render.push_plain_line(
            format!("-- bevim: {} --", self.display_name),
            Color::srgb(0.9, 0.85, 0.5),
        );

        for (idx, _) in self.buffer.iter().enumerate() {
            let line_bg =
                (highlight_line == Some(idx + 1)).then_some(active_line_highlight_color());
            let mut line = StyledLine::default();
            line.push_segment(format!("{:>4} ", idx + 1), Color::srgb(0.6, 0.6, 0.9));
            let segments_line = if self.highlight {
                self.syntax_highlight_line(idx, cursor_visible, line_bg)
            } else {
                self.plain_line(idx, cursor_visible, line_bg)
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

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    fn plain_line(&self, row: usize, cursor_visible: bool, line_bg: Option<Color>) -> StyledLine {
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
            line_bg,
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

    fn syntax_highlight_line(
        &self,
        row: usize,
        cursor_visible: bool,
        line_bg: Option<Color>,
    ) -> StyledLine {
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
                    line_bg,
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
                    line_bg,
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
                    &mut line, chars, start, idx, word_color, cursor_col, cursor_bg, line_bg,
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
                    line_bg,
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
                line_bg,
            );
            idx += 1;
        }

        if matches!(cursor_col, Some(col) if col == chars.len()) {
            line.push_segment_with_bg(" ", Color::srgb(0.9, 0.9, 0.9), Some(cursor_bg));
        }

        line
    }

    #[allow(clippy::too_many_arguments)]
    fn push_token_with_cursor(
        &self,
        line: &mut StyledLine,
        chars: &[char],
        start: usize,
        end: usize,
        color: Color,
        cursor_col: Option<usize>,
        cursor_bg: Color,
        line_bg: Option<Color>,
    ) {
        if start >= end {
            return;
        }
        if let Some(cursor) = cursor_col
            && cursor >= start
            && cursor < end
        {
            if cursor > start {
                let prefix = chars[start..cursor].iter().collect::<String>();
                Self::push_colored_segment(line, prefix, color, line_bg);
            }
            let mut cursor_char = String::new();
            cursor_char.push(chars[cursor]);
            line.push_segment_with_bg(cursor_char, color, Some(cursor_bg));
            if cursor + 1 < end {
                let suffix = chars[cursor + 1..end].iter().collect::<String>();
                Self::push_colored_segment(line, suffix, color, line_bg);
            }
            return;
        }
        let text = chars[start..end].iter().collect::<String>();
        Self::push_colored_segment(line, text, color, line_bg);
    }

    fn push_colored_segment(
        line: &mut StyledLine,
        text: String,
        color: Color,
        background: Option<Color>,
    ) {
        if text.is_empty() {
            return;
        }
        if let Some(bg) = background {
            line.push_segment_with_bg(text, color, Some(bg));
        } else {
            line.push_segment(text, color);
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(line) = self.buffer.get_mut(self.cursor_row) {
            line.insert(self.cursor_col, ch);
            self.cursor_col += 1;
        }
    }

    pub fn insert_newline(&mut self) {
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

    pub fn backspace(&mut self) {
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

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.buffer[self.cursor_row].len();
        }
    }

    pub fn move_right(&mut self) {
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

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    pub fn move_down(&mut self) {
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

    pub fn enter_insert_mode(&mut self) {
        self.mode = VimMode::Insert;
        self.status = "INSERT mode — Esc to return to NORMAL".into();
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = VimMode::Command;
        self.command_buffer.clear();
        self.status = "COMMAND mode — type :w, :wq, or :q".into();
    }

    pub fn enter_normal_mode(&mut self, message: &str) {
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
        self.status = message.into();
    }

    pub fn push_command_char(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    pub fn submit_command(&mut self) -> VimCommandAction {
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

    pub fn set_status(&mut self, status: String) {
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

pub enum VimMode {
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

pub enum VimCommandAction {
    None,
    Save { quit: bool },
    Quit,
}

#[derive(Default)]
pub struct ShellState {
    lines: Vec<String>,
    current_input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    saved_input: Option<String>,
}

impl ShellState {
    pub fn new() -> Self {
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

    pub fn render(&self, focused: bool, cursor_visible: bool) -> TerminalRender {
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

    pub fn push_char(&mut self, ch: char) {
        self.current_input.push(ch);
    }

    pub fn backspace(&mut self) {
        self.current_input.pop();
    }

    pub fn finish_command(&mut self) -> String {
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

    pub fn push_history(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    pub fn clear_history(&mut self) {
        self.lines.clear();
        self.reset_lines();
        self.current_input.clear();
        self.history_index = None;
        self.saved_input = None;
    }

    pub fn history_previous(&mut self) -> bool {
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

    pub fn history_next(&mut self) -> bool {
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

    pub fn autocomplete(&mut self, commands: &[&str]) -> bool {
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

pub fn live_root_path() -> &'static Path {
    Path::new(super::LIVE_ROOT)
}

pub fn sanitize_live_target(target: &str) -> Result<PathBuf, String> {
    let trimmed = target.trim();
    let fallback = if trimmed.is_empty() {
        super::DEFAULT_FILE
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
        relative.push(super::DEFAULT_FILE);
    }
    if relative.extension().is_none() {
        relative.set_extension("mortar");
    }
    Ok(relative)
}

fn cursor_highlight_color() -> Color {
    Color::srgb(0.25, 0.4, 0.9)
}

fn active_line_highlight_color() -> Color {
    Color::srgb(0.2, 0.2, 0.25)
}

pub fn save_buffer(editor: &VimEditorState) -> std::io::Result<PathBuf> {
    let path = live_root_path().join(&editor.relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, editor.to_string())?;
    Ok(path)
}

pub fn build_editor_for_target(target: &str) -> Result<VimEditorState, String> {
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
