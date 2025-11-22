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
mod system;

pub use asset::{MortarAsset, MortarAssetLoader};
pub use bevy_mortar_bond_macros::{MortarFunctions, mortar_functions};
pub use binder::{
    MortarBoolean, MortarFunctionRegistry, MortarNumber, MortarString, MortarValue, MortarVoid,
};

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
    pub functions: binder::MortarFunctionRegistry,
}

impl Default for MortarRuntime {
    fn default() -> Self {
        Self {
            active_dialogue: None,
            pending_start: None,
            pending_jump: None,
            functions: binder::MortarFunctionRegistry::new(),
        }
    }
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
    choice_stack: Vec<usize>,
    /// A snapshot of the node data (to avoid repeated queries).
    ///
    /// 节点数据的快照（避免重复查询）。
    node_data: Node,
}

impl DialogueState {
    /// Creates a new dialogue state.
    ///
    /// 创建一个新的对话状态。
    pub fn new(mortar_path: String, node_name: String, node_data: Node) -> Self {
        Self {
            mortar_path,
            current_node: node_name,
            text_index: 0,
            selected_choice: None,
            choice_stack: Vec::new(),
            node_data,
        }
    }

    /// Gets the current choices, considering the choice stack for nested selections.
    ///
    /// 获取当前选择，考虑嵌套选择的堆栈。
    pub fn get_current_choices(&self) -> Option<&Vec<Choice>> {
        let mut choices = self.node_data.choice.as_ref()?;

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
        if self.choice_stack.is_empty() {
            self.node_data.choice.as_ref()
        } else {
            self.get_current_choices()
        }
    }

    /// Gets the currently displayed text (raw text without interpolation).
    ///
    /// 获取当前显示的文本（不含插值的原始文本）。
    pub fn current_text(&self) -> Option<&str> {
        self.node_data
            .texts
            .get(self.text_index)
            .map(|s| s.text.as_str())
    }

    /// Gets the current text data with interpolation information.
    ///
    /// 获取包含插值信息的当前文本数据。
    pub fn current_text_data(&self) -> Option<&mortar_compiler::Text> {
        self.node_data.texts.get(self.text_index)
    }

    /// Checks if there is more text to display.
    ///
    /// 检查是否还有更多文本。
    pub fn has_next_text(&self) -> bool {
        self.text_index + 1 < self.node_data.texts.len()
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

    /// Checks if the current node has choices.
    ///
    /// 检查当前节点是否有选项。
    pub fn has_choices(&self) -> bool {
        self.node_data.choice.is_some()
    }

    /// Gets the next node name (for automatic jumps).
    ///
    /// 获取下一个节点名称（用于自动跳转）。
    pub fn get_next_node(&self) -> Option<&str> {
        self.node_data.next.as_deref()
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

/// Evaluates a condition by calling the bound function.
///
/// 通过调用绑定函数来评估条件。
pub fn evaluate_condition(
    condition: &mortar_compiler::Condition,
    functions: &binder::MortarFunctionRegistry,
    function_decls: &[mortar_compiler::Function],
) -> bool {
    // Parse arguments
    let args: Vec<binder::MortarValue> = condition
        .args
        .iter()
        .map(|arg| binder::MortarValue::parse(arg))
        .collect();

    // Call the function
    if let Some(value) = functions.call(&condition.condition_type, &args) {
        // Try to convert to boolean
        match value {
            binder::MortarValue::Boolean(b) => b.0,
            binder::MortarValue::Number(n) => n.0 != 0.0,
            binder::MortarValue::String(s) => !s.0.is_empty(),
            binder::MortarValue::Void => false,
        }
    } else {
        // Function not found - default to false
        warn!(
            "Condition function '{}' not bound, defaulting to false",
            condition.condition_type
        );
        false
    }
}

/// Processes interpolated text by calling bound functions.
///
/// 通过调用绑定函数来处理插值文本。
pub fn process_interpolated_text(
    text_data: &mortar_compiler::Text,
    functions: &binder::MortarFunctionRegistry,
    function_decls: &[mortar_compiler::Function],
) -> String {
    // If there are no interpolated parts, return the original text
    let Some(parts) = &text_data.interpolated_parts else {
        return text_data.text.clone();
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
                    let args: Vec<binder::MortarValue> = part
                        .args
                        .iter()
                        .map(|arg| binder::MortarValue::parse(arg))
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
                // For placeholders, just keep the content as-is
                result.push_str(&part.content);
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
