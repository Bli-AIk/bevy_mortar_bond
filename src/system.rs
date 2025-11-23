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
                let Some(state) = &mut runtime.active_dialogue else {
                    continue;
                };

                // Mark the content slot right after the current text for run execution
                state.pending_run_position = state
                    .current_text_content_index()
                    .map(|content_idx| content_idx + 1);

                if state.next_text() {
                    continue;
                }

                dev_info!("Reached end of node: {}", state.current_node);

                // Check if has choices and choices are not broken
                if state.has_choices() && !state.choices_broken {
                    dev_info!("Node has choices, waiting for user selection");
                    continue;
                }

                if let Some(next_node) = state.get_next_node() {
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
            MortarEvent::SelectChoice { index } => {
                let Some(state) = &mut runtime.active_dialogue else {
                    warn!("No active dialogue to select choice from");
                    continue;
                };
                let Some(choices) = state.get_choices() else {
                    warn!("No choices available in current node");
                    continue;
                };
                if *index >= choices.len() {
                    warn!("Invalid choice index: {}", index);
                    continue;
                }

                dev_info!(
                    "Choice marked as selected: {} - {}",
                    index,
                    choices[*index].text
                );
                state.selected_choice = Some(*index);
            }
            MortarEvent::ConfirmChoice => {
                let Some(state) = &mut runtime.active_dialogue else {
                    warn!("No active dialogue to confirm choice from");
                    continue;
                };
                let Some(choice_index) = state.selected_choice else {
                    warn!("No choice selected to confirm");
                    continue;
                };
                let Some(choices) = state.get_choices() else {
                    warn!("No choices available in current node");
                    continue;
                };
                let Some(choice) = choices.get(choice_index) else {
                    warn!("Invalid choice index: {}", choice_index);
                    continue;
                };

                dev_info!("Choice confirmed: {} - {}", choice_index, choice.text);

                if let Some(action) = &choice.action {
                    match action.as_str() {
                        "return" => {
                            dev_info!("Choice action is return, stopping dialogue");
                            runtime.active_dialogue = None;
                            continue;
                        }
                        "break" => {
                            dev_info!("Choice action is break, continuing to next text");
                            // Clear the choice stack and selection
                            state.clear_choice_stack();
                            // Mark choices as broken so they won't be shown anymore
                            state.choices_broken = true;
                            // Advance to next text
                            state.next_text();
                            continue;
                        }
                        _ => {
                            dev_info!("Unknown choice action: {}", action);
                            runtime.active_dialogue = None;
                            continue;
                        }
                    }
                }

                // Check if this choice has nested choices
                if choice.choice.is_some() {
                    dev_info!("Choice has nested choices, entering nested level");
                    state.push_choice(choice_index);
                    continue;
                }

                if let Some(next_node) = &choice.next {
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
    let Some((path, node)) = runtime.pending_start.clone() else {
        return;
    };
    let Some(handle) = registry.get(&path) else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };
    let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == node) else {
        return;
    };

    let state = DialogueState::new(path.clone(), node.clone(), node_data.clone());
    runtime.active_dialogue = Some(state);
    runtime.pending_start = None;
    dev_info!("Started pending node: {} in {}", node, path);
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
