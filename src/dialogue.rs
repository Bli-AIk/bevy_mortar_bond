use crate::{
    DialogueRunKind, MortarAsset, MortarAudioSettings, MortarEvent, MortarEventTracker,
    MortarRegistry, MortarRuntime, MortarVariableState, MortarVariableValue,
    audio::auto_play_sound_events, evaluate_if_condition, process_interpolated_text,
};
use bevy::asset::Assets;
use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Plugin that turns Mortar runtime data into ready-to-render UI text plus gameplay events.
///
/// 负责把 Mortar 运行时数据转换成可直接渲染的文本和可监听的游戏事件。
pub struct MortarDialoguePlugin;

/// System sets exposed by [`MortarDialoguePlugin`] for ordering customization.
///
/// [`MortarDialoguePlugin`] 暴露的系统集合，方便自定义执行顺序。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MortarDialogueSystemSet {
    /// Systems that update dialogue text output.
    ///
    /// 更新对话文本输出的系统。
    UpdateText,
    /// Systems that emit gameplay events based on bound indices.
    ///
    /// 基于绑定索引发出游戏事件的系统。
    TriggerEvents,
}

impl Plugin for MortarDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MortarDialogueSystemSet::UpdateText,
                MortarDialogueSystemSet::TriggerEvents,
            )
                .chain(),
        )
        .init_resource::<MortarAudioSettings>()
        .init_resource::<MortarDialogueVariables>()
        .init_resource::<MortarRunsExecuting>()
        .init_resource::<LoggedConstants>()
        .add_message::<MortarGameEvent>()
        .add_systems(
            Update,
            (
                log_public_constants_once,
                process_run_statements_after_text,
                update_mortar_text_targets.in_set(MortarDialogueSystemSet::UpdateText),
                trigger_bound_events.in_set(MortarDialogueSystemSet::TriggerEvents),
                process_pending_run_executions,
                auto_play_sound_events
                    .after(MortarDialogueSystemSet::TriggerEvents)
                    .after(process_pending_run_executions),
            ),
        )
        .add_systems(PostUpdate, clear_runs_executing_flag);
    }
}

/// Marker for `Text` entities that should display Mortar dialogue output.
///
/// 标记需要显示 Mortar 对话文本的 UI 实体。
#[derive(Component)]
pub struct MortarTextTarget;

/// Stores the current Mortar dialogue text so users can bind custom render effects.
///
/// 存储当前 Mortar 对话文本，便于绑定自定义渲染效果。
#[derive(Component, Debug, Clone, Default)]
pub struct MortarDialogueText {
    /// Prefix header string (`[file / node]`).
    ///
    /// 前缀头字符串（`[文件 / 节点]`）。
    pub header: String,
    /// Body text processed from Mortar.
    ///
    /// Mortar 处理后的正文文本。
    pub body: String,
}

impl MortarDialogueText {
    /// Returns the concatenated header + body string.
    ///
    /// 返回拼接后的头部与正文文本。
    pub fn full_text(&self) -> String {
        format!("{}{}", self.header, self.body)
    }
}

/// Component that exposes the current playback index for Mortar events.
///
/// 用户可以将 `current_index` 绑定到任意系统（打字机、
/// 音频时间线等），由 [`MortarDialoguePlugin`] 自动触发事件。
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MortarEventBinding {
    /// Progress index used by [`MortarEventTracker`].
    ///
    /// [`MortarEventTracker`] 使用的进度索引。
    pub current_index: f32,
}

/// Event emitted whenever Mortar timelines or text events ask the game to do something.
///
/// 由 Mortar 文本事件或时间线触发的游戏事件。
#[derive(Message, Debug, Clone)]
pub struct MortarGameEvent {
    /// The logical source entity (usually the dialogue text). `None` for timeline-only events.
    ///
    /// 逻辑来源实体（通常是对话文本）；仅时间线事件时为 `None`。
    pub source: Option<Entity>,
    /// Event name defined inside Mortar (e.g. "set_animation").
    ///
    /// Mortar 中定义的事件名称（如 "set_animation"）。
    pub name: String,
    /// Raw argument list from Mortar.
    ///
    /// 来自 Mortar 的原始参数列表。
    pub args: Vec<String>,
}

/// Resource that caches variable state for the currently loaded mortar file.
///
/// 缓存当前 mortar 文件变量状态的资源。
#[derive(Resource, Default)]
pub struct MortarDialogueVariables {
    pub(crate) state: Option<MortarVariableState>,
    active_path: Option<String>,
}

impl MortarDialogueVariables {
    fn reset(&mut self) {
        self.state = None;
        self.active_path = None;
    }

    fn ensure_for(
        &mut self,
        path: &str,
        asset: &mortar_compiler::MortaredData,
    ) -> &mut MortarVariableState {
        if self.active_path.as_deref() != Some(path) {
            self.state = Some(MortarVariableState::from_variables(
                &asset.variables,
                &asset.enums,
            ));
            self.active_path = Some(path.to_string());
        } else if self.state.is_none() {
            self.state = Some(MortarVariableState::from_variables(
                &asset.variables,
                &asset.enums,
            ));
        }
        self.state.as_mut().expect("variable state initialized")
    }
}

/// Tracks whether Mortar `run` statements are executing.
///
/// 记录 `run` 语句是否正在执行。
#[derive(Resource, Default)]
pub struct MortarRunsExecuting {
    pub executing: bool,
}

/// Records which mortar files already printed their public constants.
///
/// 记录哪些 mortar 文件已经打印过其公共常量。
#[derive(Resource, Default)]
pub struct LoggedConstants {
    seen_paths: HashSet<String>,
}

/// Component that schedules pending run/timeline execution with timers.
///
/// 使用计时器安排待执行 run 或时间线的组件。
#[derive(Component)]
struct PendingRunExecution {
    timer: Timer,
    remaining_runs: Vec<(String, Option<f64>, bool)>,
    event_defs: Vec<mortar_compiler::EventDef>,
    timeline_defs: Vec<mortar_compiler::TimelineDef>,
}

#[derive(SystemParam)]
struct TextUpdateParams<'w, 's> {
    commands: Commands<'w, 's>,
    runtime: Res<'w, MortarRuntime>,
    registry: Res<'w, MortarRegistry>,
    assets: Res<'w, Assets<MortarAsset>>,
    texts: Query<'w, 's, (Entity, &'static mut Text), With<MortarTextTarget>>,
    variable_cache: ResMut<'w, MortarDialogueVariables>,
    runs_executing: Res<'w, MortarRunsExecuting>,
    events: MessageWriter<'w, MortarEvent>,
}

fn log_public_constants_once(
    runtime: Res<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    mut logged: ResMut<LoggedConstants>,
) {
    let Some(state) = &runtime.active_dialogue else {
        return;
    };

    if logged.seen_paths.contains(&state.mortar_path) {
        return;
    }

    let Some(handle) = registry.get(&state.mortar_path) else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };

    let public_consts: Vec<_> = asset.data.constants.iter().filter(|c| c.public).collect();
    if public_consts.is_empty() {
        logged.seen_paths.insert(state.mortar_path.clone());
        return;
    }

    info!("Mortar public constants exposed by {}:", state.mortar_path);
    for constant in public_consts {
        let value_repr = match &constant.value {
            serde_json::Value::String(s) => s.clone(),
            _ => constant.value.to_string(),
        };
        info!(
            "  {} ({}): {}",
            constant.name, constant.const_type, value_repr
        );
    }

    logged.seen_paths.insert(state.mortar_path.clone());
}

fn update_mortar_text_targets(
    params: TextUpdateParams,
    mut last_key: Local<Option<(String, String, usize)>>,
    mut skip_next_conditional: Local<bool>,
) {
    let TextUpdateParams {
        mut commands,
        runtime,
        registry,
        assets,
        mut texts,
        mut variable_cache,
        runs_executing,
        mut events,
    } = params;

    if runs_executing.executing {
        return;
    }

    if runtime.active_dialogue.is_none() {
        variable_cache.reset();
        for (_, mut text) in &mut texts {
            **text = "等待加载对话...".to_string();
        }
        *last_key = None;
        return;
    }

    if !runtime.is_changed() {
        return;
    }

    for (entity, mut text) in &mut texts {
        let Some(state) = &runtime.active_dialogue else {
            **text = "等待加载对话...".to_string();
            *last_key = None;
            continue;
        };

        let asset_data = registry
            .get(&state.mortar_path)
            .and_then(|handle| assets.get(handle))
            .map(|asset| &asset.data);

        let current_key = (
            state.mortar_path.clone(),
            state.current_node.clone(),
            state.text_index,
        );

        if last_key.as_ref() == Some(&current_key) {
            continue;
        }

        let Some(text_data) = state.current_text_data() else {
            continue;
        };

        let variable_state = if let Some(asset_data) = asset_data {
            variable_cache.ensure_for(&state.mortar_path, asset_data)
        } else {
            variable_cache
                .state
                .get_or_insert_with(MortarVariableState::new)
        };

        if *skip_next_conditional && text_data.condition.is_some() {
            *skip_next_conditional = false;
            *last_key = Some(current_key);
            events.write(MortarEvent::NextText);
            continue;
        }

        if let Some(condition) = &text_data.condition
            && !evaluate_if_condition(condition, &runtime.functions, variable_state)
        {
            *skip_next_conditional = false;
            *last_key = Some(current_key.clone());
            events.write(MortarEvent::NextText);
            continue;
        }

        let mut executed_statements = false;
        for stmt in &text_data.pre_statements {
            if stmt.stmt_type == "assignment"
                && let (Some(var_name), Some(value)) = (&stmt.var_name, &stmt.value)
            {
                variable_state.execute_assignment(var_name, value);
                executed_statements = true;
            }
        }

        let processed_text = process_interpolated_text(
            text_data,
            &runtime.functions,
            asset_data
                .map(|data| data.functions.as_slice())
                .unwrap_or(&[]),
            variable_state,
        );

        if processed_text.is_empty() {
            if executed_statements && text_data.condition.is_some() {
                *skip_next_conditional = true;
            }
            *last_key = Some(current_key);
            events.write(MortarEvent::NextText);
            continue;
        }

        *skip_next_conditional = false;

        commands.entity(entity).remove::<MortarEventTracker>();
        commands.entity(entity).remove::<MortarEventBinding>();

        let all_events = collect_text_events(
            text_data,
            variable_state,
            asset_data,
            state.current_text_content_index(),
            state.node_data(),
        );

        if !all_events.is_empty() {
            commands
                .entity(entity)
                .insert(MortarEventTracker::new(all_events))
                .insert(MortarEventBinding::default());
        }

        *last_key = Some(current_key);

        let header = format!("[{} / {}]\n\n", state.mortar_path, state.current_node);
        let final_text = format!("{}{}", header, processed_text);
        **text = final_text.clone();
        commands.entity(entity).insert(MortarDialogueText {
            header,
            body: processed_text,
        });
    }
}

fn collect_text_events(
    text_data: &crate::TextData,
    variable_state: &MortarVariableState,
    asset_data: Option<&mortar_compiler::MortaredData>,
    current_text_content_idx: Option<usize>,
    node_data: &mortar_compiler::Node,
) -> Vec<mortar_compiler::Event> {
    let mut all_events = Vec::new();

    if let Some(parts) = &text_data.interpolated_parts {
        if let Some(asset_data) = asset_data {
            let mut original_pos = 0.0;
            let mut rendered_pos = 0.0;
            let mut index_map = HashMap::new();

            for part in parts {
                if part.part_type == "text" {
                    for _ in 0..part.content.chars().count() {
                        index_map.insert(original_pos as usize, rendered_pos);
                        original_pos += 1.0;
                        rendered_pos += 1.0;
                    }
                } else if part.part_type == "placeholder" {
                    let var_name = part.content.trim_matches(|c| c == '{' || c == '}');

                    if let Some(branch_events) =
                        variable_state.get_branch_events(var_name, &asset_data.variables)
                    {
                        for mut event in branch_events {
                            event.index += rendered_pos;
                            all_events.push(event);
                        }
                    }

                    if let Some(branch_text) = variable_state.get_branch_text(var_name) {
                        rendered_pos += branch_text.chars().count() as f64;
                    } else if let Some(value) = variable_state.get(var_name) {
                        rendered_pos += value.to_display_string().chars().count() as f64;
                    }
                }
            }

            index_map.insert(original_pos as usize, rendered_pos);

            if let Some(text_events) = &text_data.events {
                for event in text_events {
                    let mut adjusted_event = event.clone();

                    if let Some(var_name) = &adjusted_event.index_variable
                        && let Some(MortarVariableValue::Number(n)) = variable_state.get(var_name)
                    {
                        adjusted_event.index = *n;
                    }

                    if let Some(&rendered_index) = index_map.get(&(adjusted_event.index as usize)) {
                        adjusted_event.index = rendered_index;
                    }

                    all_events.push(adjusted_event);
                }
            }
        }
    } else if let Some(text_events) = &text_data.events {
        all_events = text_events.clone();
        for event in &mut all_events {
            if let Some(var_name) = &event.index_variable
                && let Some(MortarVariableValue::Number(n)) = variable_state.get(var_name)
            {
                event.index = *n;
            }
        }
    }

    if let Some(asset_data) = asset_data
        && let Some(content_idx) = current_text_content_idx
        && let Some(prev_content) = content_idx
            .checked_sub(1)
            .and_then(|idx| node_data.content.get(idx))
        && let Some("run_event") = prev_content.get("type").and_then(|v| v.as_str())
        && let Some(index_override) = prev_content
            .get("index_override")
            .and_then(|v| serde_json::from_value::<mortar_compiler::IndexOverride>(v.clone()).ok())
        && let Some(event_name) = prev_content.get("name").and_then(|v| v.as_str())
    {
        let index = if index_override.override_type == "variable" {
            variable_state
                .get(&index_override.value)
                .and_then(|v| {
                    if let MortarVariableValue::Number(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0)
        } else {
            index_override.value.parse::<f64>().unwrap_or(0.0)
        };

        if let Some(event_def) = asset_data.events.iter().find(|e| e.name == event_name) {
            let text_event = mortar_compiler::Event {
                index,
                index_variable: None,
                actions: vec![event_def.action.clone()],
            };
            all_events.push(text_event);
        }
    }

    all_events
}

fn trigger_bound_events(
    mut query: Query<(Entity, &MortarEventBinding, &mut MortarEventTracker)>,
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

fn process_run_statements_after_text(
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

    let Some(state) = &mut runtime.active_dialogue else {
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
        state.pending_run_position = None;
        return;
    }

    let Some(handle) = registry.get(&state.mortar_path) else {
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

    if let Some(state) = &mut runtime.active_dialogue {
        for idx in content_indices_to_mark {
            state.mark_content_executed(idx);
        }
        state.pending_run_position = None;
    }
}

fn process_pending_run_executions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PendingRunExecution)>,
    mut runtime: ResMut<MortarRuntime>,
    mut runs_executing: ResMut<MortarRunsExecuting>,
    mut game_events: MessageWriter<MortarGameEvent>,
) {
    for (entity, mut pending) in &mut query {
        pending.timer.tick(time.delta());

        if pending.timer.just_finished() {
            if pending.remaining_runs.is_empty() {
                commands.entity(entity).despawn();
                runs_executing.executing = false;
                if runtime.active_dialogue.is_some() {
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
                if let Some(_state) = &runtime.active_dialogue {
                    runtime.set_changed();
                }
            }
        }
    }
}

fn clear_runs_executing_flag(
    pending_runs_query: Query<&PendingRunExecution>,
    mut runs_executing: ResMut<MortarRunsExecuting>,
    mut runtime: ResMut<MortarRuntime>,
) {
    if runs_executing.executing && pending_runs_query.is_empty() {
        runs_executing.executing = false;
        if runtime.active_dialogue.is_some() {
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

    if let Some(timeline_def) = timeline_defs.iter().find(|t| t.name == event_name) {
        let mut timeline_sequence = Vec::new();
        for stmt in &timeline_def.statements {
            match stmt.stmt_type.as_str() {
                "run" => {
                    if let Some(event_name) = &stmt.event_name {
                        let duration = if stmt.ignore_duration {
                            None
                        } else {
                            stmt.duration
                        };
                        timeline_sequence.push((
                            event_name.clone(),
                            duration,
                            stmt.ignore_duration,
                        ));
                    }
                }
                "wait" => {
                    if let Some(duration) = stmt.duration {
                        timeline_sequence.push(("__WAIT__".to_string(), Some(duration), false));
                    }
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
        return false;
    }

    warn!("Run statement target not found: {}", event_name);
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
