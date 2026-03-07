use crate::{
    DialogueState, MortarAsset, MortarDialogueFinished, MortarEvent, MortarRegistry, MortarRuntime,
};
use bevy::asset::{AssetServer, Assets};
use bevy::log::{info, warn};
use bevy::prelude::{Entity, MessageReader, MessageWriter, Res, ResMut};

fn entity_to_option(entity: Entity) -> Option<Entity> {
    (entity != Entity::PLACEHOLDER).then_some(entity)
}

fn remove_entity_dialogue(runtime: &mut MortarRuntime, entity: Entity) {
    runtime.active_dialogues.remove(&entity);
    if runtime.primary_dialogue == Some(entity) {
        runtime.primary_dialogue = None;
    }
}

fn handle_start_node(
    path: &str,
    node: &str,
    target: Option<Entity>,
    runtime: &mut MortarRuntime,
    registry: &mut MortarRegistry,
    assets: &Assets<MortarAsset>,
    asset_server: &AssetServer,
) {
    let handle = if let Some(h) = registry.get(path) {
        h.clone()
    } else {
        info!("Auto-loading mortar file: {}", path);
        let handle = asset_server.load::<MortarAsset>(path.to_owned());
        registry.register(path.to_owned(), handle.clone());
        handle
    };

    let Some(asset) = assets.get(&handle) else {
        dev_info!("Asset '{}' not loaded yet, waiting...", path);
        let entity = target.unwrap_or(Entity::PLACEHOLDER);
        runtime
            .pending_starts
            .insert(entity, (path.to_owned(), node.to_owned()));
        return;
    };
    let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == node) else {
        warn!("Node '{}' not found in '{}'", node, path);
        return;
    };
    let state = DialogueState::new(path.to_owned(), node.to_owned(), node_data.clone());

    let entity = target.unwrap_or(Entity::PLACEHOLDER);
    runtime.active_dialogues.insert(entity, state);
    runtime.primary_dialogue = Some(entity);
    runtime.pending_starts.remove(&entity);
    dev_info!("Started node: {} in {} for entity {:?}", node, path, entity);
}

fn handle_next_text(
    target: Option<Entity>,
    runtime: &mut MortarRuntime,
    finished_events: &mut MessageWriter<MortarDialogueFinished>,
) {
    let Some(entity) = target.or(runtime.primary_dialogue) else {
        return;
    };

    let (should_continue, has_choices, choices_broken, next_node_info, mortar_path, current_node) = {
        let Some(state) = runtime.active_dialogues.get_mut(&entity) else {
            return;
        };
        state.pending_run_position = state
            .current_text_content_index()
            .map(|content_idx| content_idx + 1);

        if state.next_text() {
            (true, false, false, None, String::new(), String::new())
        } else {
            dev_info!("Reached end of node: {}", state.current_node);
            let has_choices = state.has_choices();
            let choices_broken = state.choices_broken;
            let next_node = state.get_next_node().map(|s| s.to_string());
            let path = state.mortar_path.clone();
            let node = state.current_node.clone();
            (false, has_choices, choices_broken, next_node, path, node)
        }
    };

    if should_continue {
        return;
    }

    if has_choices && !choices_broken {
        dev_info!("Node has choices, waiting for user selection");
        return;
    }

    let Some(next_node) = next_node_info else {
        dev_info!("Node ended without next or choices for entity {:?}", entity);
        remove_entity_dialogue(runtime, entity);
        finished_events.write(MortarDialogueFinished {
            entity: entity_to_option(entity),
            mortar_path,
            node: current_node,
        });
        return;
    };

    if next_node == "return" {
        dev_info!(
            "Return instruction, stopping dialogue for entity {:?}",
            entity
        );
        remove_entity_dialogue(runtime, entity);
        finished_events.write(MortarDialogueFinished {
            entity: entity_to_option(entity),
            mortar_path,
            node: current_node,
        });
    } else {
        dev_info!("Auto-jumping to next node: {}", next_node);
        runtime
            .pending_jumps
            .insert(entity, (mortar_path, next_node));
    }
}

fn handle_select_choice(index: usize, target: Option<Entity>, runtime: &mut MortarRuntime) {
    let Some(entity) = target.or(runtime.primary_dialogue) else {
        warn!("No active dialogue to select choice from");
        return;
    };
    let Some(state) = runtime.active_dialogues.get_mut(&entity) else {
        warn!("No active dialogue for entity {:?}", entity);
        return;
    };
    let Some(choices) = state.get_choices() else {
        warn!("No choices available in current node");
        return;
    };
    if index >= choices.len() {
        warn!("Invalid choice index: {}", index);
        return;
    }

    dev_info!(
        "Choice marked as selected: {} - {}",
        index,
        choices[index].text
    );
    state.selected_choice = Some(index);
}

fn handle_choice_action(
    action: &str,
    entity: Entity,
    runtime: &mut MortarRuntime,
    finished_events: &mut MessageWriter<MortarDialogueFinished>,
    mortar_path: &str,
    current_node: &str,
) {
    match action {
        "return" => {
            dev_info!("Choice action is return, stopping dialogue");
            remove_entity_dialogue(runtime, entity);
            finished_events.write(MortarDialogueFinished {
                entity: entity_to_option(entity),
                mortar_path: mortar_path.to_owned(),
                node: current_node.to_owned(),
            });
        }
        "break" => {
            dev_info!("Choice action is break, continuing to next text");
            let Some(state) = runtime.active_dialogues.get_mut(&entity) else {
                return;
            };
            state.clear_choice_stack();
            state.choices_broken = true;
            state.next_text();
        }
        _ => {
            dev_info!("Unknown choice action: {}", action);
            remove_entity_dialogue(runtime, entity);
            finished_events.write(MortarDialogueFinished {
                entity: entity_to_option(entity),
                mortar_path: mortar_path.to_owned(),
                node: current_node.to_owned(),
            });
        }
    }
}

fn handle_confirm_choice(
    target: Option<Entity>,
    runtime: &mut MortarRuntime,
    finished_events: &mut MessageWriter<MortarDialogueFinished>,
) {
    let Some(entity) = target.or(runtime.primary_dialogue) else {
        warn!("No active dialogue to confirm choice from");
        return;
    };

    let (choice_index, choices_clone, mortar_path, current_node) = {
        let Some(state) = runtime.active_dialogues.get(&entity) else {
            warn!("No active dialogue for entity {:?}", entity);
            return;
        };
        let Some(choice_index) = state.selected_choice else {
            warn!("No choice selected to confirm");
            return;
        };
        let Some(choices) = state.get_choices() else {
            warn!("No choices available in current node");
            return;
        };
        (
            choice_index,
            choices.clone(),
            state.mortar_path.clone(),
            state.current_node.clone(),
        )
    };

    let Some(choice) = choices_clone.get(choice_index) else {
        warn!("Invalid choice index: {}", choice_index);
        return;
    };

    dev_info!("Choice confirmed: {} - {}", choice_index, choice.text);

    if let Some(action) = &choice.action {
        handle_choice_action(
            action,
            entity,
            runtime,
            finished_events,
            &mortar_path,
            &current_node,
        );
        return;
    }

    if choice.choice.is_some() {
        dev_info!("Choice has nested choices, entering nested level");
        if let Some(state) = runtime.active_dialogues.get_mut(&entity) {
            state.push_choice(choice_index);
        }
        return;
    }

    let Some(next_node) = &choice.next else {
        dev_info!("Choice has no next node or action, stopping dialogue");
        remove_entity_dialogue(runtime, entity);
        finished_events.write(MortarDialogueFinished {
            entity: entity_to_option(entity),
            mortar_path,
            node: current_node,
        });
        return;
    };

    if next_node == "return" {
        dev_info!("Choice leads to return, stopping dialogue");
        remove_entity_dialogue(runtime, entity);
        finished_events.write(MortarDialogueFinished {
            entity: entity_to_option(entity),
            mortar_path,
            node: current_node,
        });
    } else {
        dev_info!("Choice leads to node: {}", next_node);
        runtime
            .pending_jumps
            .insert(entity, (mortar_path, next_node.clone()));
    }
}

fn handle_stop_dialogue(target: Option<Entity>, runtime: &mut MortarRuntime) {
    let Some(entity) = target else {
        runtime.active_dialogues.clear();
        runtime.pending_starts.clear();
        runtime.pending_jumps.clear();
        runtime.primary_dialogue = None;
        dev_info!("All dialogues stopped");
        return;
    };
    runtime.active_dialogues.remove(&entity);
    runtime.pending_starts.remove(&entity);
    runtime.pending_jumps.remove(&entity);
    if runtime.primary_dialogue == Some(entity) {
        runtime.primary_dialogue = None;
    }
    dev_info!("Dialogue stopped for entity {:?}", entity);
}

/// Processes Mortar events.
/// Now supports multi-controller architecture with optional target entities.
///
/// 处理 Mortar 事件。
/// 现在支持多控制器架构和可选的目标实体。
pub fn process_mortar_events_system(
    mut events: MessageReader<MortarEvent>,
    mut runtime: ResMut<MortarRuntime>,
    mut registry: ResMut<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    asset_server: Res<AssetServer>,
    mut finished_events: MessageWriter<MortarDialogueFinished>,
) {
    for event in events.read() {
        match event {
            MortarEvent::StartNode { path, node, target } => handle_start_node(
                path,
                node,
                *target,
                &mut runtime,
                &mut registry,
                &assets,
                &asset_server,
            ),
            MortarEvent::NextText { target } => {
                handle_next_text(*target, &mut runtime, &mut finished_events)
            }
            MortarEvent::SelectChoice { index, target } => {
                handle_select_choice(*index, *target, &mut runtime)
            }
            MortarEvent::ConfirmChoice { target } => {
                handle_confirm_choice(*target, &mut runtime, &mut finished_events)
            }
            MortarEvent::StopDialogue { target } => handle_stop_dialogue(*target, &mut runtime),
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
    // Collect entities to process (avoid borrowing issues)
    let pending: Vec<(Entity, String, String)> = runtime
        .pending_starts
        .iter()
        .map(|(e, (p, n))| (*e, p.clone(), n.clone()))
        .collect();

    for (entity, path, node) in pending {
        let Some(handle) = registry.get(&path) else {
            continue;
        };
        let Some(asset) = assets.get(handle) else {
            continue;
        };
        let Some(node_data) = asset.data.nodes.iter().find(|n| n.name == node) else {
            continue;
        };

        let state = DialogueState::new(path.clone(), node.clone(), node_data.clone());
        runtime.active_dialogues.insert(entity, state);
        runtime.primary_dialogue = Some(entity);
        runtime.pending_starts.remove(&entity);
        dev_info!(
            "Started pending node: {} in {} for entity {:?}",
            node,
            path,
            entity
        );
    }
}

/// Handles pending jumps to other nodes.
///
/// 处理等待中的节点跳转。
pub fn handle_pending_jump_system(
    mut runtime: ResMut<MortarRuntime>,
    mut event_writer: MessageWriter<MortarEvent>,
) {
    // Collect pending jumps to process
    let jumps: Vec<(Entity, String, String)> = runtime
        .pending_jumps
        .drain()
        .map(|(e, (p, n))| (e, p, n))
        .collect();

    for (entity, path, node) in jumps {
        dev_info!(
            "Processing pending jump to: {} in {} for entity {:?}",
            node,
            path,
            entity
        );
        event_writer.write(MortarEvent::StartNode {
            path,
            node,
            target: Some(entity),
        });
    }
}
