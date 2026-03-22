//! # run_execution.rs
//!
//! # run_execution.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file executes `run_event` and `run_timeline` entries that appear inside Mortar dialogue
//! content. It schedules delayed runs, dispatches gameplay events at the right time, and keeps the
//! dialogue runtime informed about when run-driven pauses begin and end.
//!
//! 这个文件负责执行 Mortar 对话内容里的 `run_event` 和 `run_timeline` 条目。它会安排带延迟
//! 的 run，按正确时机分发游戏事件，并让对话运行时知道由 run 驱动的暂停何时开始和结束。

use bevy::asset::Assets;
use bevy::prelude::*;
use std::time::Duration;

use crate::{DialogueRunKind, MortarAsset, MortarRegistry, MortarRuntime};

use super::{MortarEventBinding, MortarGameEvent, MortarRunsExecuting, MortarTextTarget};

/// Component that schedules pending run/timeline execution with timers.
///
/// 使用计时器安排待执行 run 或时间线的组件。
#[derive(Component)]
pub(super) struct PendingRunExecution {
    timer: Timer,
    remaining_runs: Vec<(String, Option<f64>, bool)>,
    event_defs: Vec<mortar_compiler::EventDef>,
    timeline_defs: Vec<mortar_compiler::TimelineDef>,
}

pub(super) fn trigger_bound_events(
    mut query: Query<(Entity, &MortarEventBinding, &mut crate::MortarEventTracker)>,
    runtime: Res<MortarRuntime>,
    mut writer: MessageWriter<MortarGameEvent>,
) {
    for (entity, binding, mut tracker) in &mut query {
        let actions = tracker.trigger_at_index(binding.current_index, &runtime);
        for action in actions {
            writer.write(MortarGameEvent {
                source: Some(entity),
                name: action.action_name,
                args: action.args,
            });
        }
    }
}

pub(super) fn process_run_statements_after_text(
    mut commands: Commands,
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    mut text_query: Query<&mut Text, With<MortarTextTarget>>,
    mut runs_executing: ResMut<MortarRunsExecuting>,
    mut game_events: MessageWriter<MortarGameEvent>,
) {
    if !runtime.is_changed() {
        return;
    }

    let Some(state) = runtime.primary_dialogue_state_mut() else {
        return;
    };

    let Some(start_search_idx) = state.pending_run_position else {
        return;
    };

    if start_search_idx >= state.node_data().content.len() {
        state.pending_run_position = None;
        return;
    }

    let run_items = state.collect_run_items_from(start_search_idx);
    let mortar_path = state.mortar_path.clone();
    let mut run_sequence = Vec::new();
    let mut content_indices_to_mark = Vec::new();

    for item in &run_items {
        match item.kind {
            DialogueRunKind::Event | DialogueRunKind::Timeline => {
                run_sequence.push((item.name.clone(), None::<f64>, item.ignore_duration));
                content_indices_to_mark.push(item.content_index);
            }
        }
    }

    if run_sequence.is_empty() {
        if let Some(state) = runtime.primary_dialogue_state_mut() {
            state.pending_run_position = None;
        }
        return;
    }

    let Some(handle) = registry.get(&mortar_path) else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };

    let event_defs = &asset.data.events;
    let timeline_defs = &asset.data.timelines;

    let run_sequence_with_durations: Vec<(String, Option<f64>, bool)> = run_sequence
        .iter()
        .map(|(name, _, ignore_duration)| {
            let duration = event_defs
                .iter()
                .find(|e| e.name == *name)
                .and_then(|e| e.duration);
            (name.clone(), duration, *ignore_duration)
        })
        .collect();

    runs_executing.executing = true;

    for mut text in &mut text_query {
        **text = String::new();
    }

    if run_sequence_with_durations.len() > 1 {
        let pending = start_timeline_execution(
            run_sequence_with_durations,
            event_defs.to_vec(),
            timeline_defs.to_vec(),
            &mut commands,
            &mut game_events,
        );
        if !pending {
            runs_executing.executing = false;
        }
    } else if let Some((event_name, _, _)) = run_sequence_with_durations.first() {
        let timeline_running = execute_run_by_name(
            event_name,
            event_defs,
            timeline_defs,
            &mut commands,
            &mut game_events,
        );

        if !timeline_running {
            runs_executing.executing = false;
        }
    }

    if let Some(state) = runtime.primary_dialogue_state_mut() {
        for idx in content_indices_to_mark {
            state.mark_content_executed(idx);
        }
        state.pending_run_position = None;
    }
}

pub(super) fn process_pending_run_executions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PendingRunExecution)>,
    mut runtime: ResMut<MortarRuntime>,
    mut runs_executing: ResMut<MortarRunsExecuting>,
    mut game_events: MessageWriter<MortarGameEvent>,
) {
    for (entity, mut pending) in &mut query {
        pending.timer.tick(time.delta());

        if !pending.timer.just_finished() {
            continue;
        }

        if pending.remaining_runs.is_empty() {
            commands.entity(entity).despawn();
            runs_executing.executing = false;
            if runtime.has_active_dialogues() {
                runtime.set_changed();
            }
            continue;
        }

        if let Some((event_name, _, _)) = pending.remaining_runs.first()
            && event_name != "__WAIT__"
            && let Some(event_def) = pending.event_defs.iter().find(|e| e.name == *event_name)
        {
            dispatch_game_event(&event_def.action, &mut game_events);
        }

        if pending.remaining_runs.len() > 1 {
            let remaining = pending.remaining_runs[1..].to_vec();
            let next_event = &pending.remaining_runs[0];

            let duration_secs = if next_event.0 == "__WAIT__" {
                next_event.1.unwrap_or(0.0)
            } else if next_event.2 {
                0.0
            } else {
                pending
                    .event_defs
                    .iter()
                    .find(|e| e.name == next_event.0)
                    .and_then(|e| e.duration)
                    .unwrap_or(0.0)
            };

            if duration_secs > 0.0 {
                pending.remaining_runs = remaining;
                pending
                    .timer
                    .set_duration(Duration::from_secs_f32(duration_secs as f32));
                pending.timer.reset();
            } else {
                let event_defs = pending.event_defs.clone();
                let timeline_defs = pending.timeline_defs.clone();
                commands.entity(entity).despawn();
                let _ = start_timeline_execution(
                    remaining,
                    event_defs,
                    timeline_defs,
                    &mut commands,
                    &mut game_events,
                );
            }
        } else {
            commands.entity(entity).despawn();
            runs_executing.executing = false;
            if runtime.has_active_dialogues() {
                runtime.set_changed();
            }
        }
    }
}

pub(super) fn clear_runs_executing_flag(
    pending_runs_query: Query<&PendingRunExecution>,
    mut runs_executing: ResMut<MortarRunsExecuting>,
    mut runtime: ResMut<MortarRuntime>,
) {
    if runs_executing.executing && pending_runs_query.is_empty() {
        runs_executing.executing = false;
        if runtime.has_active_dialogues() {
            runtime.set_changed();
        }
    }
}

fn execute_run_by_name(
    event_name: &str,
    event_defs: &[mortar_compiler::EventDef],
    timeline_defs: &[mortar_compiler::TimelineDef],
    commands: &mut Commands,
    game_events: &mut MessageWriter<MortarGameEvent>,
) -> bool {
    if let Some(event_def) = event_defs.iter().find(|e| e.name == event_name) {
        dispatch_game_event(&event_def.action, game_events);
        return false;
    }

    let Some(timeline_def) = timeline_defs.iter().find(|t| t.name == event_name) else {
        warn!("Run statement target not found: {}", event_name);
        return false;
    };

    let mut timeline_sequence = Vec::new();
    for stmt in &timeline_def.statements {
        match stmt.stmt_type.as_str() {
            "run" => {
                let Some(event_name) = &stmt.event_name else {
                    continue;
                };
                let duration = stmt.duration.filter(|_| !stmt.ignore_duration);
                timeline_sequence.push((event_name.clone(), duration, stmt.ignore_duration));
            }
            "wait" => {
                let Some(duration) = stmt.duration else {
                    continue;
                };
                timeline_sequence.push(("__WAIT__".to_string(), Some(duration), false));
            }
            _ => {}
        }
    }

    if !timeline_sequence.is_empty() {
        let _ = start_timeline_execution(
            timeline_sequence,
            event_defs.to_vec(),
            timeline_defs.to_vec(),
            commands,
            game_events,
        );
        return true;
    }
    false
}

fn start_timeline_execution(
    sequence: Vec<(String, Option<f64>, bool)>,
    event_defs: Vec<mortar_compiler::EventDef>,
    timeline_defs: Vec<mortar_compiler::TimelineDef>,
    commands: &mut Commands,
    game_events: &mut MessageWriter<MortarGameEvent>,
) -> bool {
    let mut spawned_async = false;

    if let Some((first_event, first_duration, ignore_duration)) = sequence.first() {
        if first_event != "__WAIT__"
            && let Some(event_def) = event_defs.iter().find(|e| e.name == *first_event)
        {
            dispatch_game_event(&event_def.action, game_events);
        }

        if sequence.len() > 1 {
            let remaining = sequence[1..].to_vec();
            let duration_secs = if first_event == "__WAIT__" {
                first_duration.unwrap_or(0.0)
            } else if *ignore_duration {
                0.0
            } else {
                event_defs
                    .iter()
                    .find(|e| e.name == *first_event)
                    .and_then(|e| e.duration)
                    .unwrap_or(0.0)
            };

            if duration_secs > 0.0 {
                commands.spawn((PendingRunExecution {
                    timer: Timer::from_seconds(duration_secs as f32, TimerMode::Once),
                    remaining_runs: remaining,
                    event_defs,
                    timeline_defs,
                },));
                spawned_async = true;
            } else {
                spawned_async |= start_timeline_execution(
                    remaining,
                    event_defs,
                    timeline_defs,
                    commands,
                    game_events,
                );
            }
        }
    }

    spawned_async
}

fn dispatch_game_event(
    action: &mortar_compiler::Action,
    events: &mut MessageWriter<MortarGameEvent>,
) {
    let parsed_args: Vec<String> = action
        .args
        .iter()
        .map(|arg| arg.trim_matches('"').to_string())
        .collect();

    events.write(MortarGameEvent {
        source: None,
        name: action.action_type.clone(),
        args: parsed_args,
    });
}
