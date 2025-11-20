use bevy::prelude::*;
use std::collections::HashMap;
use mortar_compiler::Node;

mod asset;

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
            .add_systems(Update, (process_mortar_events, check_pending_start));
    }
}

/// 全局 Mortar 资源注册表，管理多个 mortar 文件
#[derive(Resource, Default)]
pub struct MortarRegistry {
    assets: HashMap<String, Handle<MortarAsset>>,
}

impl MortarRegistry {
    /// 注册一个 mortar 资源，使用路径名作为标识符
    pub fn register(&mut self, path: impl Into<String>, handle: Handle<MortarAsset>) {
        self.assets.insert(path.into(), handle);
    }

    /// 获取已注册的资源句柄
    pub fn get(&self, path: &str) -> Option<&Handle<MortarAsset>> {
        self.assets.get(path)
    }
}

/// Mortar 运行时状态
#[derive(Resource, Default)]
pub struct MortarRuntime {
    /// 当前激活的对话状态
    pub active_dialogue: Option<DialogueState>,
    /// 等待启动的节点 (path, node)
    pub pending_start: Option<(String, String)>,
}

/// 对话状态
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// mortar 文件路径
    pub mortar_path: String,
    /// 当前节点名称
    pub current_node: String,
    /// 当前文本索引
    pub text_index: usize,
    /// 节点数据的快照（避免重复查询）
    node_data: Node,
}

impl DialogueState {
    pub fn new(mortar_path: String, node_name: String, node_data: Node) -> Self {
        Self {
            mortar_path,
            current_node: node_name,
            text_index: 0,
            node_data,
        }
    }

    /// 获取当前显示的文本
    pub fn current_text(&self) -> Option<&str> {
        self.node_data.texts.get(self.text_index).map(|s| s.text.as_str())
    }

    /// 是否还有更多文本
    pub fn has_next_text(&self) -> bool {
        self.text_index + 1 < self.node_data.texts.len()
    }

    /// 步进到下一条文本
    pub fn next_text(&mut self) -> bool {
        if self.has_next_text() {
            self.text_index += 1;
            true
        } else {
            false
        }
    }

    /// 重置到节点开始
    pub fn reset(&mut self) {
        self.text_index = 0;
    }
}

/// Mortar 事件系统
#[derive(Message, Debug, Clone)]
pub enum MortarEvent {
    /// 启动一个节点：(mortar_path, node_name)
    StartNode { path: String, node: String },
    /// 步进到下一条文本
    NextText,
    /// 选择一个选项
    SelectChoice { index: usize },
    /// 停止当前对话
    StopDialogue,
}

/// 处理 Mortar 事件
fn process_mortar_events(
    mut events: MessageReader<MortarEvent>,
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
) {
    for event in events.read() {
        match event {
            MortarEvent::StartNode { path, node } => {
                if let Some(handle) = registry.get(path) {
                    if let Some(asset) = assets.get(handle) {
                        if let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == *node) {
                            let state = DialogueState::new(
                                path.clone(),
                                node.clone(),
                                node_data.clone(),
                            );
                            runtime.active_dialogue = Some(state);
                            runtime.pending_start = None;
                            info!("Started node: {} in {}", node, path);
                        } else {
                            warn!("Node '{}' not found in '{}'", node, path);
                        }
                    } else {
                        info!("Asset '{}' not loaded yet, waiting...", path);
                        runtime.pending_start = Some((path.clone(), node.clone()));
                    }
                } else {
                    warn!("Mortar path '{}' not registered", path);
                }
            }
            MortarEvent::NextText => {
                if let Some(state) = &mut runtime.active_dialogue {
                    if !state.next_text() {
                        info!("Reached end of node: {}", state.current_node);
                        // 可以在这里处理节点结束逻辑，比如跳转到其他节点
                    }
                }
            }
            MortarEvent::SelectChoice { index } => {
                // TODO: 实现选项选择逻辑
                info!("Choice selected: {}", index);
            }
            MortarEvent::StopDialogue => {
                runtime.active_dialogue = None;
                runtime.pending_start = None;
                info!("Dialogue stopped");
            }
        }
    }
}

/// 检查并启动等待中的节点
fn check_pending_start(
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
) {
    if let Some((path, node)) = runtime.pending_start.clone() {
        if let Some(handle) = registry.get(&path) {
            if let Some(asset) = assets.get(handle) {
                if let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == node) {
                    let state = DialogueState::new(
                        path.clone(),
                        node.clone(),
                        node_data.clone(),
                    );
                    runtime.active_dialogue = Some(state);
                    runtime.pending_start = None;
                    info!("Started pending node: {} in {}", node, path);
                }
            }
        }
    }
}

