//! # dialogue_state.rs
//!
//! # dialogue_state.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the in-memory dialogue state machine used by `bevy_mortar_bond`. It parses a
//! Mortar node into text, choice, and run-oriented views, then stores the cursor, executed content
//! markers, pending runs, and choice navigation data needed while a dialogue is active.
//!
//! 定义了 `bevy_mortar_bond` 使用的内存对话状态机。它会把 Mortar 节点拆成面向文本、
//! 选项和 run 的视图，并保存对话进行中所需的游标、已执行内容标记、待执行 run 以及选项导航数据。

use bevy::prelude::*;
use mortar_compiler::{Choice, Node};

/// Text data extracted from content item
///
/// 从内容项提取的文本数据
#[derive(Debug, Clone)]
pub struct TextData {
    pub value: String,
    pub interpolated_parts: Option<Vec<mortar_compiler::StringPart>>,
    pub condition: Option<mortar_compiler::IfCondition>,
    pub pre_statements: Vec<mortar_compiler::Statement>,
    pub events: Option<Vec<mortar_compiler::Event>>,
    /// When true, consecutive lines are joined with `\n` into a single display unit.
    pub is_line: bool,
}

/// The state of a dialogue.
///
/// 对话状态。
#[derive(Debug, Clone)]
pub struct DialogueState {
    pub mortar_path: String,
    pub current_node: String,
    pub text_index: usize,
    pub selected_choice: Option<usize>,
    pub choice_stack: Vec<usize>,
    pub choices_broken: bool,
    pub executed_content_indices: Vec<usize>,
    pub pending_run_position: Option<usize>,
    node_data: Node,
    text_items: Vec<TextData>,
    text_to_content_index: Vec<usize>,
    choice_content_index: Option<usize>,
    choices: Option<Vec<Choice>>,
}

/// Type of run content embedded in a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogueRunKind {
    Event,
    Timeline,
}

/// Metadata describing run_event / run_timeline entries in node content.
#[derive(Debug, Clone)]
pub struct DialogueRunItem {
    pub content_index: usize,
    pub name: String,
    pub kind: DialogueRunKind,
    pub ignore_duration: bool,
}

/// Descriptor for run statements found at a specific content position.
pub type DialogueRunDescriptor = (
    usize,
    String,
    Vec<String>,
    Option<mortar_compiler::IndexOverride>,
    bool,
);

fn parse_node_content(
    content_idx: usize,
    content_value: &serde_json::Value,
    text_items: &mut Vec<TextData>,
    text_to_content_index: &mut Vec<usize>,
    choice_content_index: &mut Option<usize>,
    choices: &mut Option<Vec<Choice>>,
) {
    let Some(type_str) = content_value.get("type").and_then(|value| value.as_str()) else {
        return;
    };
    match type_str {
        "text" | "line" => {
            let is_line = type_str == "line";
            let value = content_value
                .get("value")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let interpolated_parts = content_value
                .get("interpolated_parts")
                .and_then(|value| serde_json::from_value(value.clone()).ok());
            let condition = content_value
                .get("condition")
                .and_then(|value| serde_json::from_value(value.clone()).ok());
            let pre_statements = content_value
                .get("pre_statements")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or_default();
            let events = content_value
                .get("events")
                .and_then(|value| serde_json::from_value(value.clone()).ok());

            text_items.push(TextData {
                value,
                interpolated_parts,
                condition,
                pre_statements,
                events,
                is_line,
            });
            text_to_content_index.push(content_idx);
        }
        "choice" => {
            let Some(options_value) = content_value.get("options") else {
                return;
            };
            let Ok(parsed_choices) = serde_json::from_value::<Vec<Choice>>(options_value.clone())
                .inspect_err(|err| {
                    warn!(
                        "Failed to parse choice options at content index {}: {}",
                        content_idx, err
                    );
                })
            else {
                return;
            };
            *choices = Some(parsed_choices);
            *choice_content_index = Some(content_idx);
        }
        _ => {}
    }
}

fn collect_consecutive_runs(
    content: &[serde_json::Value],
    start_index: usize,
    executed: &[usize],
) -> Vec<DialogueRunItem> {
    let mut runs = Vec::new();
    for (idx, content_value) in content.iter().enumerate().skip(start_index) {
        if executed.contains(&idx) {
            continue;
        }
        let Some(type_str) = content_value.get("type").and_then(|value| value.as_str()) else {
            break;
        };
        match type_str {
            "run_event" if content_value.get("index_override").is_some() => continue,
            "run_event" => {
                let Some(name) = content_value.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                let ignore_duration = content_value
                    .get("ignore_duration")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                runs.push(DialogueRunItem {
                    content_index: idx,
                    name: name.to_string(),
                    kind: DialogueRunKind::Event,
                    ignore_duration,
                });
            }
            "run_timeline" => {
                let Some(name) = content_value.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                runs.push(DialogueRunItem {
                    content_index: idx,
                    name: name.to_string(),
                    kind: DialogueRunKind::Timeline,
                    ignore_duration: false,
                });
            }
            _ => break,
        }
    }
    runs
}

fn collect_runs_at_position(
    content: &[serde_json::Value],
    content_position: usize,
    executed: &[usize],
) -> Vec<DialogueRunDescriptor> {
    let mut runs = Vec::new();
    for (idx, content_value) in content.iter().enumerate() {
        if idx != content_position || executed.contains(&idx) {
            continue;
        }
        let Some(type_str) = content_value.get("type").and_then(|value| value.as_str()) else {
            continue;
        };
        match type_str {
            "run_event" => {
                let Some(name) = content_value.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                let args = content_value
                    .get("args")
                    .and_then(|value| serde_json::from_value(value.clone()).ok())
                    .unwrap_or_default();
                let index_override = content_value
                    .get("index_override")
                    .and_then(|value| serde_json::from_value(value.clone()).ok());
                let ignore_duration = content_value
                    .get("ignore_duration")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                runs.push((idx, name.to_string(), args, index_override, ignore_duration));
            }
            "run_timeline" => {
                let Some(name) = content_value.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                runs.push((idx, name.to_string(), vec![], None, false));
            }
            _ => {}
        }
    }
    runs
}

impl DialogueState {
    pub fn new(mortar_path: String, node_name: String, node_data: Node) -> Self {
        let mut text_items = Vec::new();
        let mut text_to_content_index = Vec::new();
        let mut choice_content_index = None;
        let mut choices = None;

        for (content_idx, content_value) in node_data.content.iter().enumerate() {
            parse_node_content(
                content_idx,
                content_value,
                &mut text_items,
                &mut text_to_content_index,
                &mut choice_content_index,
                &mut choices,
            );
        }

        Self {
            mortar_path,
            current_node: node_name,
            text_index: 0,
            selected_choice: None,
            choice_stack: Vec::new(),
            choices_broken: false,
            executed_content_indices: Vec::new(),
            pending_run_position: None,
            node_data,
            text_items,
            text_to_content_index,
            choice_content_index,
            choices,
        }
    }

    pub fn get_current_choices(&self) -> Option<&Vec<Choice>> {
        let mut choices = self.choices.as_ref()?;
        for &index in &self.choice_stack {
            let choice = choices.get(index)?;
            let nested = choice.choice.as_ref()?;
            choices = nested;
        }
        Some(choices)
    }

    pub fn push_choice(&mut self, index: usize) {
        self.choice_stack.push(index);
        self.selected_choice = None;
    }

    pub fn pop_choice(&mut self) -> Option<usize> {
        self.selected_choice = None;
        self.choice_stack.pop()
    }

    pub fn clear_choice_stack(&mut self) {
        self.choice_stack.clear();
        self.selected_choice = None;
    }

    pub fn get_choices(&self) -> Option<&Vec<Choice>> {
        if self.choices_broken {
            return None;
        }

        if self.choice_stack.is_empty() {
            self.choices.as_ref()
        } else {
            self.get_current_choices()
        }
    }

    pub fn current_text(&self) -> Option<&str> {
        self.text_items
            .get(self.text_index)
            .map(|text| text.value.as_str())
    }

    pub fn current_text_data(&self) -> Option<&TextData> {
        self.text_items.get(self.text_index)
    }

    pub fn current_text_data_evaluated(
        &self,
        variable_state: &crate::MortarVariableState,
        functions: &crate::MortarFunctionRegistry,
    ) -> Option<&TextData> {
        let text_data = self.text_items.get(self.text_index)?;

        if text_data.condition.is_none() {
            return Some(text_data);
        }

        if let Some(condition) = &text_data.condition {
            return if crate::evaluate_if_condition(condition, functions, variable_state) {
                Some(text_data)
            } else {
                None
            };
        }

        Some(text_data)
    }

    fn line_group_end(&self) -> usize {
        let Some(current) = self.text_items.get(self.text_index) else {
            return self.text_index + 1;
        };
        if !current.is_line {
            return self.text_index + 1;
        }
        let mut end = self.text_index + 1;
        while end < self.text_items.len() && self.text_items[end].is_line {
            end += 1;
        }
        end
    }

    pub fn current_line_group(&self) -> Option<&[TextData]> {
        self.text_items.get(self.text_index)?;
        Some(&self.text_items[self.text_index..self.line_group_end()])
    }

    pub fn has_next_text(&self) -> bool {
        self.line_group_end() < self.text_items.len()
    }

    pub fn has_next_text_before_choice(&self) -> bool {
        if let Some(choice_content_idx) = self.choice_content_index {
            let next_idx = self.line_group_end();
            if next_idx < self.text_items.len() {
                let next_text_content_idx = self.text_to_content_index[next_idx];
                next_text_content_idx < choice_content_idx
            } else {
                false
            }
        } else {
            self.has_next_text()
        }
    }

    pub fn next_text(&mut self) -> bool {
        let end = self.line_group_end();
        if end < self.text_items.len() {
            self.text_index = end;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.text_index = 0;
    }

    pub fn collect_run_items_from(&self, start_index: usize) -> Vec<DialogueRunItem> {
        collect_consecutive_runs(
            &self.node_data.content,
            start_index,
            &self.executed_content_indices,
        )
    }

    pub fn has_choices(&self) -> bool {
        self.choices.is_some()
    }

    pub fn get_next_node(&self) -> Option<&str> {
        self.node_data.next.as_deref()
    }

    pub fn get_runs_at_content_position(
        &self,
        content_position: usize,
    ) -> Vec<DialogueRunDescriptor> {
        collect_runs_at_position(
            &self.node_data.content,
            content_position,
            &self.executed_content_indices,
        )
    }

    pub fn mark_content_executed(&mut self, content_index: usize) {
        if !self.executed_content_indices.contains(&content_index) {
            self.executed_content_indices.push(content_index);
        }
    }

    pub fn node_data(&self) -> &Node {
        &self.node_data
    }

    pub fn current_text_content_index(&self) -> Option<usize> {
        self.text_to_content_index.get(self.text_index).copied()
    }

    pub fn line_group_last_content_index(&self) -> Option<usize> {
        let end = self.line_group_end();
        self.text_to_content_index.get(end - 1).copied()
    }

    pub fn text_to_content_indices(&self) -> &[usize] {
        &self.text_to_content_index
    }
}
