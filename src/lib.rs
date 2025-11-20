//! This crate provides a 'bond' (binding) system for the Mortar dialogue system, integrating it with the Bevy game engine.
//!
//! 本包为 Mortar 对话系统提供“绑钉”（绑定）系统，将其与 Bevy 游戏引擎集成。

use bevy::prelude::*;
use mortar_compiler::Node;
use std::collections::HashMap;

#[macro_use]
mod debug;
mod asset;
mod system;

pub use asset::{MortarAsset, MortarAssetLoader};

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
                ),
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
#[derive(Resource, Default)]
pub struct MortarRuntime {
    /// The currently active dialogue state.
    ///
    /// 当前激活的对话状态。
    pub active_dialogue: Option<DialogueState>,
    /// A node that is pending to be started (path, node).
    ///
    /// 等待启动的节点 (path, node)。
    pub pending_start: Option<(String, String)>,
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
            node_data,
        }
    }

    /// Gets the currently displayed text.
    ///
    /// 获取当前显示的文本。
    pub fn current_text(&self) -> Option<&str> {
        self.node_data
            .texts
            .get(self.text_index)
            .map(|s| s.text.as_str())
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
    /// Selects a choice.
    ///
    /// 选择一个选项。
    SelectChoice { index: usize },
    /// Stops the current dialogue.
    ///
    /// 停止当前对话。
    StopDialogue,
}
