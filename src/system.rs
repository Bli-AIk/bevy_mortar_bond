use crate::{DialogueState, MortarAsset, MortarEvent, MortarRegistry, MortarRuntime};
use bevy::asset::Assets;
use bevy::log::{info, warn};
use bevy::prelude::{MessageReader, Res, ResMut};

/// Processes Mortar events.
///
/// 处理 Mortar 事件。
pub fn process_mortar_events_system(
    mut events: MessageReader<MortarEvent>,
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
) {
    for event in events.read() {
        match event {
            MortarEvent::StartNode { path, node } => {
                let Some(handle) = registry.get(path) else {
                    warn!("Mortar path '{}' not registered", path);
                    continue;
                };
                let Some(asset) = assets.get(handle) else {
                    info!("Asset '{}' not loaded yet, waiting...", path);
                    runtime.pending_start = Some((path.clone(), node.clone()));
                    continue;
                };
                let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == *node) else {
                    warn!("Node '{}' not found in '{}'", node, path);
                    continue;
                };
                let state = DialogueState::new(path.clone(), node.clone(), node_data.clone());
                runtime.active_dialogue = Some(state);
                runtime.pending_start = None;
                dev_info!("Started node: {} in {}", node, path);
            }
            MortarEvent::NextText => {
                if let Some(state) = &mut runtime.active_dialogue
                    && !state.next_text()
                {
                    dev_info!("Reached end of node: {}", state.current_node);
                    // TODO: Handle the logic of ending a node here, such as jumping to other nodes.
                    // TODO: 在这里处理节点结束逻辑，比如跳转到其他节点。
                }
            }
            MortarEvent::SelectChoice { index } => {
                // TODO: Implement choice selection logic.
                // TODO: 实现选项选择逻辑。
                dev_info!("Choice selected: {}", index);
            }
            MortarEvent::StopDialogue => {
                runtime.active_dialogue = None;
                runtime.pending_start = None;
                dev_info!("Dialogue stopped");
            }
        }
    }
}

/// Checks for and starts pending nodes.
///
/// 检查并启动等待中的节点。
pub fn check_pending_start_system(
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
) {
    if let Some((path, node)) = runtime.pending_start.clone()
        && let Some(handle) = registry.get(&path)
        && let Some(asset) = assets.get(handle)
        && let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == node)
    {
        let state = DialogueState::new(path.clone(), node.clone(), node_data.clone());
        runtime.active_dialogue = Some(state);
        runtime.pending_start = None;
        dev_info!("Started pending node: {} in {}", node, path);
    }
}
