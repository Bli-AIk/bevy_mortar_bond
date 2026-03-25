//! # events.rs
//!
//! # events.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the message types used to drive and observe Mortar dialogue playback. It
//! includes the input events sent into the runtime, the finish notification emitted at natural
//! completion, and the tracker that converts Mortar-authored text events into runtime actions.
//!
//! 定义了驱动和观测 Mortar 对话播放所用的消息类型。它包括送入运行时的输入事件、
//! 自然结束时发出的完成通知，以及把 Mortar 文本事件转换成运行时动作的跟踪器。

use bevy::prelude::*;

/// The event system for Mortar.
/// Events without a target entity operate on the primary dialogue.
#[derive(Message, Debug, Clone)]
pub enum MortarEvent {
    StartNode {
        path: String,
        node: String,
        target: Option<Entity>,
    },
    NextText {
        target: Option<Entity>,
    },
    SelectChoice {
        index: usize,
        target: Option<Entity>,
    },
    ConfirmChoice {
        target: Option<Entity>,
    },
    StopDialogue {
        target: Option<Entity>,
    },
}

impl MortarEvent {
    pub fn start_node(path: impl Into<String>, node: impl Into<String>) -> Self {
        Self::StartNode {
            path: path.into(),
            node: node.into(),
            target: None,
        }
    }

    pub fn start_node_for(
        entity: Entity,
        path: impl Into<String>,
        node: impl Into<String>,
    ) -> Self {
        Self::StartNode {
            path: path.into(),
            node: node.into(),
            target: Some(entity),
        }
    }

    pub fn next_text() -> Self {
        Self::NextText { target: None }
    }

    pub fn next_text_for(entity: Entity) -> Self {
        Self::NextText {
            target: Some(entity),
        }
    }

    pub fn stop_dialogue() -> Self {
        Self::StopDialogue { target: None }
    }

    pub fn stop_dialogue_for(entity: Entity) -> Self {
        Self::StopDialogue {
            target: Some(entity),
        }
    }
}

/// Event emitted when a Mortar dialogue finishes naturally (not via StopDialogue).
#[derive(Message, Debug, Clone)]
pub struct MortarDialogueFinished {
    pub entity: Option<Entity>,
    pub mortar_path: String,
    pub node: String,
}

fn fire_events(
    events: &[mortar_compiler::Event],
    fired_events: &mut Vec<usize>,
    current_index: f64,
    functions: &crate::MortarFunctionRegistry,
) -> Vec<MortarEventAction> {
    let mut actions_to_process = Vec::new();
    for (event_idx, event) in events.iter().enumerate() {
        if current_index < event.index || fired_events.contains(&event_idx) {
            continue;
        }
        fired_events.push(event_idx);

        debug!(
            "Mortar event triggered at index {}: {:?}",
            event.index, event.actions
        );

        for action in &event.actions {
            let args: Vec<crate::MortarValue> = action
                .args
                .iter()
                .map(|arg| crate::MortarValue::parse(arg))
                .collect();

            if let Some(result) = functions.call(&action.action_type, &args) {
                debug!(
                    "Event function '{}' returned: {:?}",
                    action.action_type, result
                );
            } else {
                warn!("Event function '{}' not found", action.action_type);
            }

            actions_to_process.push(MortarEventAction {
                action_name: action.action_type.clone(),
                args: action.args.clone(),
            });
        }
    }
    actions_to_process
}

/// Component to track mortar text events and their firing state.
#[derive(Component, Debug, Clone)]
pub struct MortarEventTracker {
    events: Vec<mortar_compiler::Event>,
    fired_events: Vec<usize>,
}

impl MortarEventTracker {
    pub fn new(events: Vec<mortar_compiler::Event>) -> Self {
        Self {
            events,
            fired_events: Vec::new(),
        }
    }

    pub fn trigger_at_index(
        &mut self,
        current_index: f32,
        runtime: &crate::MortarRuntime,
    ) -> Vec<MortarEventAction> {
        fire_events(
            &self.events,
            &mut self.fired_events,
            current_index as f64,
            &runtime.functions,
        )
    }

    pub fn reset(&mut self) {
        self.fired_events.clear();
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn fired_count(&self) -> usize {
        self.fired_events.len()
    }
}

/// An action triggered by a mortar event.
#[derive(Debug, Clone)]
pub struct MortarEventAction {
    pub action_name: String,
    pub args: Vec<String>,
}
