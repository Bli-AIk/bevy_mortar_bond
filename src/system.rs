use crate::{DialogueState, MortarAsset, MortarEvent, MortarRegistry, MortarRuntime};
use bevy::asset::Assets;
use bevy::log::{info, warn};
use bevy::prelude::{MessageReader, MessageWriter, Res, ResMut};

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

                    // Check if there are choices
                    if state.has_choices() {
                        dev_info!("Node has choices, waiting for user selection");
                    } else if let Some(next_node) = state.get_next_node() {
                        // Automatic jump to next node
                        if next_node == "return" {
                            dev_info!("Return instruction, stopping dialogue");
                            runtime.active_dialogue = None;
                        } else {
                            dev_info!("Auto-jumping to next node: {}", next_node);
                            let path = state.mortar_path.clone();
                            runtime.pending_jump = Some((path, next_node.to_string()));
                        }
                    } else {
                        dev_info!("Node ended without next or choices");
                        runtime.active_dialogue = None;
                    }
                }
            }
            MortarEvent::SelectChoice { index } => {
                if let Some(state) = &runtime.active_dialogue {
                    if let Some(choices) = state.get_choices() {
                        if let Some(choice) = choices.get(*index) {
                            dev_info!("Choice selected: {} - {}", index, choice.text);

                            // Check action field first (for return)
                            if let Some(action) = &choice.action {
                                if action == "return" {
                                    dev_info!("Choice action is return, stopping dialogue");
                                    runtime.active_dialogue = None;
                                } else {
                                    dev_info!("Unknown choice action: {}", action);
                                    runtime.active_dialogue = None;
                                }
                            }
                            // Then check next field for node jumps
                            else if let Some(next_node) = &choice.next {
                                if next_node == "return" {
                                    dev_info!("Choice leads to return, stopping dialogue");
                                    runtime.active_dialogue = None;
                                } else {
                                    dev_info!("Choice leads to node: {}", next_node);
                                    let path = state.mortar_path.clone();
                                    runtime.pending_jump = Some((path, next_node.clone()));
                                }
                            } else {
                                dev_info!("Choice has no next node or action, stopping dialogue");
                                runtime.active_dialogue = None;
                            }
                        } else {
                            warn!("Invalid choice index: {}", index);
                        }
                    } else {
                        warn!("No choices available in current node");
                    }
                } else {
                    warn!("No active dialogue to select choice from");
                }
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

/// Handles pending jumps to other nodes.
///
/// 处理等待中的节点跳转。
pub fn handle_pending_jump_system(
    mut runtime: ResMut<MortarRuntime>,
    mut event_writer: MessageWriter<MortarEvent>,
) {
    if let Some((path, node)) = runtime.pending_jump.take() {
        dev_info!("Processing pending jump to: {} in {}", node, path);
        event_writer.write(MortarEvent::StartNode { path, node });
    }
}
