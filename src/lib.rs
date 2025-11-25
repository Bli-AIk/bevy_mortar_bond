//! This crate provides a 'bond' (binding) system for the Mortar dialogue system, integrating it with the Bevy game engine.
//!
//! 本包为 Mortar 对话系统提供“绑钉”（绑定）系统，将其与 Bevy 游戏引擎集成。
//!
//! # ECS Architecture
//!
//! This library follows ECS (Entity Component System) design principles.
//!
//! ## Core Components
//!
//! - [`MortarEventTracker`]: Tracks text events and manages firing state based on index
//!
//! ## Usage Pattern
//!
//! 1. Add [`MortarEventTracker`] to entities with text events
//! 2. Call `trigger_at_index()` with current progress index
//! 3. Handle returned [`MortarEventAction`]s in your game systems
//!

use bevy::prelude::*;
use mortar_compiler::{Choice, Node};
use std::collections::HashMap;

#[macro_use]
mod debug;
mod asset;
mod binder;
mod dialogue;
mod system;
mod variable_state;

#[cfg(test)]
mod tests;

pub use asset::{MortarAsset, MortarAssetLoader};
pub use bevy_mortar_bond_macros::{MortarFunctions, mortar_functions};
pub use binder::{
    MortarBoolean, MortarFunctionRegistry, MortarNumber, MortarString, MortarValue, MortarVoid,
};
pub use dialogue::{
    MortarDialoguePlugin, MortarDialogueVariables, MortarGameEvent, MortarRunsExecuting,
    MortarTextTarget,
};
pub use variable_state::{MortarVariableState, MortarVariableValue};

/// Re-export mortar_compiler types for convenience.
///
/// 为方便使用，重新导出 mortar_compiler 类型。
pub use mortar_compiler::Event as MortarTextEvent;

/// The main plugin for the mortar 'bond' (bind) system.
///
/// Mortar "绑钉" （绑定）系统的主要插件。
pub struct MortarPlugin;

impl Plugin for MortarPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MortarAsset>()
            .init_asset_loader::<MortarAssetLoader>()
            .init_resource::<MortarRegistry>()
            .init_resource::<MortarRuntime>()
            .add_message::<MortarEvent>()
            .add_systems(
                Update,
                (
                    system::process_mortar_events_system,
                    system::check_pending_start_system,
                    system::handle_pending_jump_system,
                )
                    .chain(),
            );
    }
}

/// A global registry for Mortar assets, managing multiple mortar files.
///
/// 全局 Mortar 资源注册表，管理多个 mortar 文件。
#[derive(Resource, Default)]
pub struct MortarRegistry {
    assets: HashMap<String, Handle<MortarAsset>>,
}

impl MortarRegistry {
    /// Registers a mortar asset, using its path as an identifier.
    ///
    /// 注册一个 mortar 资源，使用路径名作为标识符。
    pub fn register(&mut self, path: impl Into<String>, handle: Handle<MortarAsset>) {
        self.assets.insert(path.into(), handle);
    }

    /// Gets the handle for a registered asset.
    ///
    /// 获取已注册的资源句柄。
    pub fn get(&self, path: &str) -> Option<&Handle<MortarAsset>> {
        self.assets.get(path)
    }
}

/// The runtime state for the Mortar system.
///
/// Mortar 运行时状态。
#[derive(Resource)]
pub struct MortarRuntime {
    /// The currently active dialogue state.
    ///
    /// 当前激活的对话状态。
    pub active_dialogue: Option<DialogueState>,
    /// A node that is pending to be started (path, node).
    ///
    /// 等待启动的节点 (path, node)。
    pub pending_start: Option<(String, String)>,
    /// Pending jump to another node (path, node).
    ///
    /// 等待跳转到另一个节点 (path, node)。
    pub pending_jump: Option<(String, String)>,
    /// The function registry for calling Mortar functions.
    ///
    /// 调用 Mortar 函数的函数注册表。
    pub functions: MortarFunctionRegistry,
}

impl Default for MortarRuntime {
    fn default() -> Self {
        Self {
            active_dialogue: None,
            pending_start: None,
            pending_jump: None,
            functions: MortarFunctionRegistry::new(),
        }
    }
}

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
}

/// The state of a dialogue.
///
/// 对话状态。
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// The path to the mortar file.
    ///
    /// mortar 文件路径。
    pub mortar_path: String,
    /// The name of the current node.
    ///
    /// 当前节点名称。
    pub current_node: String,
    /// The index of the current text.
    ///
    /// 当前文本索引。
    pub text_index: usize,
    /// The currently selected choice (if any).
    ///
    /// 当前选中的选项（如果有）。
    pub selected_choice: Option<usize>,
    /// Stack of nested choice indices to track nested selections.
    ///
    /// 嵌套选择索引的堆栈，用于跟踪嵌套选择。
    pub choice_stack: Vec<usize>,
    /// Flag to indicate that choices have been broken and should not be shown.
    ///
    /// 标志表示选项已被 break，不应再显示。
    pub choices_broken: bool,
    /// Track which content items have been executed (by content index).
    ///
    /// 追踪哪些内容项已经被执行（通过内容索引）。
    pub executed_content_indices: Vec<usize>,
    /// Position to execute runs at (set by NextText handler).
    ///
    /// 要执行run的位置（由NextText处理器设置）。
    pub pending_run_position: Option<usize>,
    /// A snapshot of the node data (to avoid repeated queries).
    ///
    /// 节点数据的快照（避免重复查询）。
    node_data: Node,
    /// Extracted text data from content for easier access
    ///
    /// 从 content 提取的文本数据，便于访问
    text_items: Vec<TextData>,
    /// Mapping from text index to content index
    ///
    /// 从文本索引到内容索引的映射
    text_to_content_index: Vec<usize>,
    /// Choice position (index in content array where choices appear)
    ///
    /// 选项位置（选项在 content 数组中出现的索引）
    choice_content_index: Option<usize>,
    /// Choices extracted from content
    ///
    /// 从 content 提取的选项
    choices: Option<Vec<Choice>>,
}

/// Type of run content embedded in a node.
///
/// 节点内的 run 内容类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogueRunKind {
    /// Calls a named event.
    ///
    /// 调用特定事件。
    Event,
    /// Executes a named timeline.
    ///
    /// 执行指定时间轴。
    Timeline,
}

/// Metadata describing run_event / run_timeline entries in node content.
///
/// 描述节点中 run_event / run_timeline 条目的元信息。
#[derive(Debug, Clone)]
pub struct DialogueRunItem {
    /// Content index in the node.
    ///
    /// 该 run 项所在的内容索引。
    pub content_index: usize,
    /// Target event or timeline name.
    ///
    /// 目标事件或时间轴名称。
    pub name: String,
    /// Item kind.
    ///
    /// 内容类型。
    pub kind: DialogueRunKind,
    /// Whether the event should ignore its declared duration.
    ///
    /// 是否忽略事件声明的 duration。
    pub ignore_duration: bool,
}

/// Descriptor for run statements found at a specific content position.
///
/// run 语句描述元组：(内容索引, 名称, 参数, index_override, ignore_duration)。
pub type DialogueRunDescriptor = (
    usize,
    String,
    Vec<String>,
    Option<mortar_compiler::IndexOverride>,
    bool,
);

impl DialogueState {
    /// Creates a new dialogue state.
    ///
    /// 创建一个新的对话状态。
    pub fn new(mortar_path: String, node_name: String, node_data: Node) -> Self {
        // Parse content array to extract texts and choices
        let mut text_items = Vec::new();
        let mut text_to_content_index = Vec::new();
        let mut choice_content_index = None;
        let mut choices = None;

        for (content_idx, content_value) in node_data.content.iter().enumerate() {
            if let Some(type_str) = content_value.get("type").and_then(|v| v.as_str()) {
                match type_str {
                    "text" => {
                        // Extract text data
                        let value = content_value
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let interpolated_parts = content_value
                            .get("interpolated_parts")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let condition = content_value
                            .get("condition")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let pre_statements = content_value
                            .get("pre_statements")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let events = content_value
                            .get("events")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());

                        text_items.push(TextData {
                            value,
                            interpolated_parts,
                            condition,
                            pre_statements,
                            events,
                        });
                        text_to_content_index.push(content_idx);
                    }
                    "choice" => {
                        // Extract choices
                        if let Some(options_value) = content_value.get("options") {
                            match serde_json::from_value::<Vec<Choice>>(options_value.clone()) {
                                Ok(parsed_choices) => {
                                    choices = Some(parsed_choices);
                                    choice_content_index = Some(content_idx);
                                }
                                Err(err) => {
                                    warn!(
                                        "Failed to parse choice options at content index {}: {}",
                                        content_idx, err
                                    );
                                }
                            }
                        }
                    }
                    "run_event" | "run_timeline" => {
                        // These will be processed directly from content when needed
                    }
                    _ => {}
                }
            }
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

    /// Gets the current choices, considering the choice stack for nested selections.
    ///
    /// 获取当前选择，考虑嵌套选择的堆栈。
    pub fn get_current_choices(&self) -> Option<&Vec<Choice>> {
        let mut choices = self.choices.as_ref()?;

        // Navigate through nested choices using the stack
        for &index in &self.choice_stack {
            if let Some(choice) = choices.get(index) {
                if let Some(nested) = &choice.choice {
                    choices = nested;
                } else {
                    return None; // Invalid path
                }
            } else {
                return None; // Invalid index
            }
        }

        Some(choices)
    }

    /// Pushes a choice index onto the stack (for entering nested choices).
    ///
    /// 将选择索引推入堆栈（用于进入嵌套选择）。
    pub fn push_choice(&mut self, index: usize) {
        self.choice_stack.push(index);
        self.selected_choice = None; // Reset selection in new level
    }

    /// Pops the last choice from the stack (for exiting nested choices).
    ///
    /// 从堆栈弹出最后一个选择（用于退出嵌套选择）。
    pub fn pop_choice(&mut self) -> Option<usize> {
        self.selected_choice = None;
        self.choice_stack.pop()
    }

    /// Clears the choice stack (for returning to top-level choices).
    ///
    /// 清空选择堆栈（用于返回顶层选择）。
    pub fn clear_choice_stack(&mut self) {
        self.choice_stack.clear();
        self.selected_choice = None;
    }

    /// Deprecated: use get_current_choices instead.
    ///
    /// 已弃用：使用 get_current_choices 代替。
    pub fn get_choices(&self) -> Option<&Vec<Choice>> {
        // If choices have been broken (by break action), don't show them
        if self.choices_broken {
            return None;
        }

        if self.choice_stack.is_empty() {
            self.choices.as_ref()
        } else {
            self.get_current_choices()
        }
    }

    /// Gets the currently displayed text (raw text without interpolation).
    ///
    /// 获取当前显示的文本（不含插值的原始文本）。
    pub fn current_text(&self) -> Option<&str> {
        self.text_items
            .get(self.text_index)
            .map(|t| t.value.as_str())
    }

    /// Gets the current text data with interpolation information.
    ///
    /// 获取包含插值信息的当前文本数据。
    pub fn current_text_data(&self) -> Option<&TextData> {
        self.text_items.get(self.text_index)
    }

    /// Gets the current text data, evaluating conditions if necessary.
    ///
    /// 获取当前文本数据，如有必要会评估条件。
    pub fn current_text_data_evaluated(
        &self,
        variable_state: &MortarVariableState,
        functions: &MortarFunctionRegistry,
    ) -> Option<&TextData> {
        // Find the appropriate text based on conditions
        let text_data = self.text_items.get(self.text_index)?;

        // If there's no condition, return the text as-is
        if text_data.condition.is_none() {
            return Some(text_data);
        }

        // If there's a condition, check it
        if let Some(condition) = &text_data.condition {
            return if evaluate_if_condition(condition, functions, variable_state) {
                Some(text_data)
            } else {
                // Condition failed, try to find else branch
                // The else branch should be the next text with no condition or matching structure
                // For now, we'll return None to skip this text
                None
            };
        }

        Some(text_data)
    }

    /// Checks if there is more text to display.
    ///
    /// 检查是否还有更多文本。
    pub fn has_next_text(&self) -> bool {
        self.text_index + 1 < self.text_items.len()
    }

    /// Checks if there is more text to display before the choice position.
    ///
    /// 检查在choice位置之前是否还有更多文本。
    pub fn has_next_text_before_choice(&self) -> bool {
        if let Some(choice_content_idx) = self.choice_content_index {
            // Check if the next text would be after the choice position
            if self.text_index + 1 < self.text_items.len() {
                let next_text_content_idx = self.text_to_content_index[self.text_index + 1];
                next_text_content_idx < choice_content_idx
            } else {
                false
            }
        } else {
            self.has_next_text()
        }
    }

    /// Advances to the next text.
    ///
    /// 步进到下一条文本。
    pub fn next_text(&mut self) -> bool {
        if self.has_next_text() {
            self.text_index += 1;
            true
        } else {
            false
        }
    }

    /// Resets the state to the beginning of the node.
    ///
    /// 重置到节点开始处。
    pub fn reset(&mut self) {
        self.text_index = 0;
    }

    /// Collects consecutive pending run statements starting at the given content index.
    ///
    /// 收集从指定内容索引开始的连续 run 语句。
    pub fn collect_run_items_from(&self, start_index: usize) -> Vec<DialogueRunItem> {
        let mut runs = Vec::new();

        for (idx, content_value) in self.node_data.content.iter().enumerate().skip(start_index) {
            if self.executed_content_indices.contains(&idx) {
                continue;
            }

            let Some(type_str) = content_value.get("type").and_then(|v| v.as_str()) else {
                break;
            };

            match type_str {
                "run_event" => {
                    // Run items with index_override are treated as text events (handled inline)
                    if content_value.get("index_override").is_some() {
                        continue;
                    }
                    if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                        let ignore_duration = content_value
                            .get("ignore_duration")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        runs.push(DialogueRunItem {
                            content_index: idx,
                            name: name.to_string(),
                            kind: DialogueRunKind::Event,
                            ignore_duration,
                        });
                    }
                }
                "run_timeline" => {
                    if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                        runs.push(DialogueRunItem {
                            content_index: idx,
                            name: name.to_string(),
                            kind: DialogueRunKind::Timeline,
                            ignore_duration: false,
                        });
                    }
                }
                _ => break,
            }
        }

        runs
    }

    /// Checks if the current node has choices.
    ///
    /// 检查当前节点是否有选项。
    pub fn has_choices(&self) -> bool {
        self.choices.is_some()
    }

    /// Gets the next node name (for automatic jumps).
    ///
    /// 获取下一个节点名称（用于自动跳转）。
    pub fn get_next_node(&self) -> Option<&str> {
        self.node_data.next.as_deref()
    }

    /// Gets run statements that should execute at the current content position.
    /// Returns (content_index, event_name, args, index_override, ignore_duration)
    ///
    /// 获取应该在当前内容位置执行的run语句。
    /// 返回 (content_index, event_name, args, index_override, ignore_duration)
    pub fn get_runs_at_content_position(
        &self,
        content_position: usize,
    ) -> Vec<DialogueRunDescriptor> {
        let mut runs = Vec::new();

        // Look for run_event and run_timeline items at the specified content position
        for (idx, content_value) in self.node_data.content.iter().enumerate() {
            if idx == content_position
                && !self.executed_content_indices.contains(&idx)
                && let Some(type_str) = content_value.get("type").and_then(|v| v.as_str())
            {
                match type_str {
                    "run_event" => {
                        if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                            let args = content_value
                                .get("args")
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                                .unwrap_or_default();
                            let index_override = content_value
                                .get("index_override")
                                .and_then(|v| serde_json::from_value(v.clone()).ok());
                            let ignore_duration = content_value
                                .get("ignore_duration")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            runs.push((
                                idx,
                                name.to_string(),
                                args,
                                index_override,
                                ignore_duration,
                            ));
                        }
                    }
                    "run_timeline" => {
                        if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                            runs.push((idx, name.to_string(), vec![], None, false));
                        }
                    }
                    _ => {}
                }
            }
        }

        runs
    }

    /// Marks a content item at the given index as executed.
    ///
    /// 标记给定索引的内容项为已执行。
    pub fn mark_content_executed(&mut self, content_index: usize) {
        if !self.executed_content_indices.contains(&content_index) {
            self.executed_content_indices.push(content_index);
        }
    }

    /// Gets the node data reference.
    ///
    /// 获取节点数据引用。
    pub fn node_data(&self) -> &Node {
        &self.node_data
    }

    /// Gets the content index for the current text.
    ///
    /// 获取当前文本的内容索引。
    pub fn current_text_content_index(&self) -> Option<usize> {
        self.text_to_content_index.get(self.text_index).copied()
    }

    /// Gets all text to content index mappings.
    ///
    /// 获取所有文本到内容索引的映射。
    pub fn text_to_content_indices(&self) -> &[usize] {
        &self.text_to_content_index
    }
}

/// The event system for Mortar.
///
/// Mortar 事件系统。
#[derive(Message, Debug, Clone)]
pub enum MortarEvent {
    /// Starts a node: (mortar_path, node_name).
    ///
    /// 启动一个节点：(mortar_path, node_name)。
    StartNode { path: String, node: String },
    /// Advances to the next text.
    ///
    /// 步进到下一条文本。
    NextText,
    /// Selects a choice (marks it as selected without confirming).
    ///
    /// 选中一个选项（标记为已选，但不确认）。
    SelectChoice { index: usize },
    /// Confirms the currently selected choice and proceeds.
    ///
    /// 确认当前选中的选项并继续。
    ConfirmChoice,
    /// Stops the current dialogue.
    ///
    /// 停止当前对话。
    StopDialogue,
}

/// Gets default return value based on type.
///
/// 根据类型获取默认返回值。
fn get_default_return_value(return_type: &str) -> String {
    match return_type {
        "Boolean" | "Bool" => "false".to_string(),
        "Number" => "0".to_string(),
        "String" => String::new(),
        _ => String::new(), // void or unknown
    }
}

/// Evaluates an IfCondition with support for function calls.
///
/// 评估 IfCondition，支持函数调用。
pub fn evaluate_if_condition(
    condition: &mortar_compiler::IfCondition,
    functions: &MortarFunctionRegistry,
    variable_state: &MortarVariableState,
) -> bool {
    match condition.cond_type.as_str() {
        "func_call" => {
            let func_name = if let Some(operand) = &condition.operand {
                operand.value.clone()
            } else {
                None
            };

            if let Some(func_name) = func_name {
                let args: Vec<MortarValue> = if let Some(right) = &condition.right {
                    if let Some(value) = &right.value {
                        value.split_whitespace().map(MortarValue::parse).collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                if let Some(value) = functions.call(&func_name, &args) {
                    value.is_truthy()
                } else {
                    warn!(
                        "Condition function '{}' not bound, defaulting to false",
                        func_name
                    );
                    false
                }
            } else {
                warn!("Function call condition missing function_name");
                false
            }
        }
        "binary" => {
            // Recursively evaluate left and right
            let left_result = evaluate_if_condition(
                condition.left.as_ref().unwrap().as_ref(),
                functions,
                variable_state,
            );
            let right_result = evaluate_if_condition(
                condition.right.as_ref().unwrap().as_ref(),
                functions,
                variable_state,
            );

            match condition.operator.as_deref() {
                Some("&&") => left_result && right_result,
                Some("||") => left_result || right_result,
                _ => {
                    // For comparison operators, delegate to variable_state
                    variable_state.evaluate_condition(condition)
                }
            }
        }
        "unary" => {
            let operand_result = evaluate_if_condition(
                condition.operand.as_ref().unwrap().as_ref(),
                functions,
                variable_state,
            );
            match condition.operator.as_deref() {
                Some("!") => !operand_result,
                _ => {
                    warn!("Unknown unary operator: {:?}", condition.operator);
                    false
                }
            }
        }
        _ => {
            // For other types, use variable_state's evaluation
            variable_state.evaluate_condition(condition)
        }
    }
}

/// Evaluates a condition by calling the bound function.
///
/// 通过调用绑定函数来评估条件。
pub fn evaluate_condition(
    condition: &mortar_compiler::Condition,
    functions: &MortarFunctionRegistry,
    _function_decls: &[mortar_compiler::Function],
) -> bool {
    // Parse arguments
    let args: Vec<MortarValue> = condition
        .args
        .iter()
        .map(|arg| MortarValue::parse(arg))
        .collect();

    // Call the function
    if let Some(value) = functions.call(&condition.condition_type, &args) {
        value.is_truthy()
    } else {
        // Function not found - default to false
        warn!(
            "Condition function '{}' not bound, defaulting to false",
            condition.condition_type
        );
        false
    }
}

/// Processes interpolated text by calling bound functions and resolving variables.
///
/// 通过调用绑定函数和解析变量来处理插值文本。
pub fn process_interpolated_text(
    text_data: &TextData,
    functions: &MortarFunctionRegistry,
    function_decls: &[mortar_compiler::Function],
    variable_state: &MortarVariableState,
) -> String {
    // If there are no interpolated parts, return the original text
    let Some(parts) = &text_data.interpolated_parts else {
        return text_data.value.clone();
    };

    let mut result = String::new();
    for part in parts {
        match part.part_type.as_str() {
            "text" => {
                result.push_str(&part.content);
            }
            "expression" => {
                // Extract function name and call it
                if let Some(func_name) = &part.function_name {
                    // Parse arguments
                    let args: Vec<MortarValue> = part
                        .args
                        .iter()
                        .map(|arg| MortarValue::parse(arg))
                        .collect();

                    // Call the function
                    if let Some(value) = functions.call(func_name, &args) {
                        result.push_str(&value.to_display_string());
                    } else {
                        // Function not found - get default value based on return type
                        let return_type = function_decls
                            .iter()
                            .find(|f| f.name == *func_name)
                            .and_then(|f| f.return_type.as_deref())
                            .unwrap_or("void");

                        let default_value = get_default_return_value(return_type);
                        warn!(
                            "Function '{}' not bound, using default return value: {}",
                            func_name, default_value
                        );
                        result.push_str(&default_value);
                    }
                } else {
                    // No function name, keep the placeholder
                    result.push_str(&part.content);
                }
            }
            "placeholder" => {
                // Extract variable name from placeholder (e.g., "{status}" -> "status")
                let var_name = part.content.trim_matches(|c| c == '{' || c == '}');

                // First try to get as a regular variable
                if let Some(value) = variable_state.get(var_name) {
                    result.push_str(&value.to_display_string());
                } else if let Some(branch_text) = variable_state.get_branch_text(var_name) {
                    // Try to get as a branch variable
                    result.push_str(&branch_text);
                } else {
                    // Variable not found, keep placeholder
                    warn!("Variable '{}' not found, keeping placeholder", var_name);
                    result.push_str(&part.content);
                }
            }
            _ => {
                // Unknown type, keep the content
                result.push_str(&part.content);
            }
        }
    }

    result
}

/// Component to track mortar text events and their firing state.
///
/// 追踪 mortar 文本事件及其触发状态的组件。
#[derive(Component, Debug, Clone)]
pub struct MortarEventTracker {
    events: Vec<mortar_compiler::Event>,
    fired_events: Vec<usize>,
}

impl MortarEventTracker {
    /// Creates a new event tracker with the given events.
    ///
    /// 使用给定的事件创建新的事件追踪器。
    pub fn new(events: Vec<mortar_compiler::Event>) -> Self {
        Self {
            events,
            fired_events: Vec::new(),
        }
    }

    /// Checks and fires events at the given index, returns actions that need processing.
    ///
    /// 检查并触发给定索引处的事件，返回需要处理的动作。
    pub fn trigger_at_index(
        &mut self,
        current_index: usize,
        runtime: &MortarRuntime,
    ) -> Vec<MortarEventAction> {
        let mut actions_to_process = Vec::new();

        for (event_idx, event) in self.events.iter().enumerate() {
            let event_index = event.index as usize;

            if current_index >= event_index && !self.fired_events.contains(&event_idx) {
                self.fired_events.push(event_idx);

                debug!(
                    "Mortar event triggered at index {}: {:?}",
                    event.index, event.actions
                );

                // Call mortar functions
                for action in &event.actions {
                    let args: Vec<MortarValue> = action
                        .args
                        .iter()
                        .map(|arg| MortarValue::parse(arg))
                        .collect();

                    if let Some(result) = runtime.functions.call(&action.action_type, &args) {
                        debug!(
                            "Event function '{}' returned: {:?}",
                            action.action_type, result
                        );
                    } else {
                        warn!("Event function '{}' not found", action.action_type);
                    }

                    // Collect actions for user to handle
                    actions_to_process.push(MortarEventAction {
                        action_name: action.action_type.clone(),
                        args: action.args.clone(),
                    });
                }
            }
        }

        actions_to_process
    }

    /// Resets the tracker, clearing all fired events.
    ///
    /// 重置追踪器，清除所有已触发的事件。
    pub fn reset(&mut self) {
        self.fired_events.clear();
    }

    /// Gets the total number of events.
    ///
    /// 获取事件总数。
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Gets the number of fired events.
    ///
    /// 获取已触发的事件数量。
    pub fn fired_count(&self) -> usize {
        self.fired_events.len()
    }
}

/// An action triggered by a mortar event.
///
/// Mortar 事件触发的动作。
#[derive(Debug, Clone)]
pub struct MortarEventAction {
    /// The name of action (e.g., "play_sound", "set_animation").
    ///
    /// 动作名称（例如 "play_sound"、"set_animation"）。
    pub action_name: String,
    /// The arguments for the action.
    ///
    /// 动作的参数。
    pub args: Vec<String>,
}
