//! # dialogue.rs
//!
//! # dialogue.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Acts as the dialogue-facing plugin surface of `bevy_mortar_bond`. It connects Mortar
//! runtime state to Bevy text entities and gameplay messages, and delegates the specialized pieces
//! such as condition caching, run execution, and text-event collection to focused helper modules.
//!
//! `bevy_mortar_bond` 面向对话层的插件入口。它把 Mortar 运行时状态连接到 Bevy
//! 文本实体和游戏消息上，并把条件缓存、run 执行和文本事件收集这些更细的工作分发给专门的辅助模块。

use crate::{
    MortarAsset, MortarAudioSettings, MortarEvent, MortarEventTracker, MortarRegistry,
    MortarRuntime, MortarVariableState, audio::auto_play_sound_events, evaluate_if_condition,
    process_interpolated_text,
};
use bevy::asset::Assets;
use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

mod condition_cache;
mod run_execution;
mod text_events;

pub use condition_cache::{CachedCondition, evaluate_condition_cached};
use text_events::collect_text_events;

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
                run_execution::process_run_statements_after_text,
                update_mortar_text_targets.in_set(MortarDialogueSystemSet::UpdateText),
                run_execution::trigger_bound_events.in_set(MortarDialogueSystemSet::TriggerEvents),
                run_execution::process_pending_run_executions,
                auto_play_sound_events
                    .after(MortarDialogueSystemSet::TriggerEvents)
                    .after(run_execution::process_pending_run_executions),
            ),
        )
        .add_systems(PostUpdate, run_execution::clear_runs_executing_flag);
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
    pub state: Option<MortarVariableState>,
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
                &asset.constants,
                &asset.enums,
            ));
            self.active_path = Some(path.to_string());
        } else if self.state.is_none() {
            self.state = Some(MortarVariableState::from_variables(
                &asset.variables,
                &asset.constants,
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
    let Some(state) = runtime.primary_dialogue_state() else {
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

/// Processes a line group: evaluates conditions per-line, processes interpolation,
/// and joins passing lines with `\n`. Returns `None` if all lines are empty/skipped.
///
/// 处理 line 组：逐行评估条件、处理插值，用 `\n` 拼接通过的行。
fn process_line_group(
    group: &[crate::TextData],
    functions: &crate::MortarFunctionRegistry,
    func_decls: &[mortar_compiler::Function],
    variable_state: &mut MortarVariableState,
) -> Option<String> {
    let mut result_lines = Vec::new();
    for line_data in group {
        if let Some(condition) = &line_data.condition
            && !evaluate_if_condition(condition, functions, variable_state)
        {
            continue;
        }
        for stmt in &line_data.pre_statements {
            if stmt.stmt_type == "assignment"
                && let (Some(var_name), Some(value)) = (&stmt.var_name, &stmt.value)
            {
                variable_state.execute_assignment(var_name, value);
            }
        }
        let line_text = process_interpolated_text(line_data, functions, func_decls, variable_state);
        if !line_text.is_empty() {
            result_lines.push(line_text);
        }
    }
    if result_lines.is_empty() {
        return None;
    }
    Some(result_lines.join("\n"))
}

fn update_mortar_text_targets(
    mut asset_events: MessageReader<AssetEvent<MortarAsset>>,
    params: TextUpdateParams,
    mut last_key: Local<Option<(String, String, usize)>>,
    mut skip_next_conditional: Local<bool>,
    mut cached_condition: Local<Option<CachedCondition>>,
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

    // Check if any Mortar asset has been modified (hot reloaded)
    for event in asset_events.read() {
        if let AssetEvent::Modified { id: _ } = event {
            // If an asset changed, force a reload of variables
            info!("Mortar asset modified, reloading variables...");
            variable_cache.reset();
            *last_key = None; // Also reset last_key to ensure text re-evaluation
            *cached_condition = None;
        }
    }

    if runs_executing.executing {
        return;
    }

    if !runtime.has_active_dialogues() {
        variable_cache.reset();
        for (_, mut text) in &mut texts {
            **text = "等待加载对话...".to_string();
        }
        *last_key = None;
        *cached_condition = None;
        return;
    }

    if !runtime.is_changed() {
        return;
    }

    for (entity, mut text) in &mut texts {
        let Some(state) = runtime.primary_dialogue_state() else {
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

        let func_decls = asset_data
            .map(|data| data.functions.as_slice())
            .unwrap_or(&[]);

        // Line groups: collect all consecutive lines, evaluate conditions per-line,
        // join passing lines with '\n'.
        //
        // Line 组：收集所有连续 line，逐行评估条件，用 '\n' 拼接通过的行。
        if text_data.is_line {
            let group = state.current_line_group().unwrap_or(&[]);
            let Some(processed_text) =
                process_line_group(group, &runtime.functions, func_decls, variable_state)
            else {
                *last_key = Some(current_key);
                events.write(MortarEvent::next_text());
                continue;
            };

            *skip_next_conditional = false;
            commands.entity(entity).remove::<MortarEventTracker>();
            commands.entity(entity).remove::<MortarEventBinding>();

            *last_key = Some(current_key);

            let header = format!("[{} / {}]\n\n", state.mortar_path, state.current_node);
            let final_text = format!("{}{}", header, processed_text);
            **text = final_text.clone();
            commands.entity(entity).insert(MortarDialogueText {
                header,
                body: processed_text,
            });
            continue;
        }

        // Regular text: handling (existing logic)
        //
        // 常规 text: 处理（现有逻辑）

        if *skip_next_conditional && text_data.condition.is_some() {
            *skip_next_conditional = false;
            *last_key = Some(current_key);
            events.write(MortarEvent::next_text());
            continue;
        }

        if let Some(condition) = &text_data.condition {
            let result = evaluate_condition_cached(
                condition,
                &runtime.functions,
                variable_state,
                &mut cached_condition,
            );
            if !result {
                *last_key = Some(current_key.clone());
                events.write(MortarEvent::next_text());
                continue;
            }
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

        let processed_text =
            process_interpolated_text(text_data, &runtime.functions, func_decls, variable_state);

        if processed_text.is_empty() {
            if executed_statements && text_data.condition.is_some() {
                *skip_next_conditional = true;
            }
            *last_key = Some(current_key);
            events.write(MortarEvent::next_text());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextData, binder::MortarFunctionRegistry};

    fn make_line(value: &str) -> TextData {
        TextData {
            value: value.to_string(),
            interpolated_parts: None,
            condition: None,
            pre_statements: vec![],
            events: None,
            is_line: true,
        }
    }

    fn make_conditional_line(value: &str, cond: mortar_compiler::IfCondition) -> TextData {
        TextData {
            value: value.to_string(),
            interpolated_parts: None,
            condition: Some(cond),
            pre_statements: vec![],
            events: None,
            is_line: true,
        }
    }

    fn true_condition() -> mortar_compiler::IfCondition {
        // A variable set to "1" evaluates to truthy
        mortar_compiler::IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("truthy_var".to_string()),
        }
    }

    fn false_condition() -> mortar_compiler::IfCondition {
        // A variable not set evaluates to falsy
        mortar_compiler::IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("unset_var".to_string()),
        }
    }

    #[test]
    fn test_process_line_group_basic() {
        let group = vec![make_line("Line A"), make_line("Line B")];
        let functions = MortarFunctionRegistry::new();
        let func_decls = vec![];
        let mut vs = MortarVariableState::default();

        let result = process_line_group(&group, &functions, &func_decls, &mut vs);
        assert_eq!(result, Some("Line A\nLine B".to_string()));
    }

    #[test]
    fn test_process_line_group_single_line() {
        let group = vec![make_line("Only line")];
        let functions = MortarFunctionRegistry::new();
        let func_decls = vec![];
        let mut vs = MortarVariableState::default();

        let result = process_line_group(&group, &functions, &func_decls, &mut vs);
        assert_eq!(result, Some("Only line".to_string()));
    }

    #[test]
    fn test_process_line_group_all_conditions_false() {
        let group = vec![
            make_conditional_line("Line A", false_condition()),
            make_conditional_line("Line B", false_condition()),
        ];
        let functions = MortarFunctionRegistry::new();
        let func_decls = vec![];
        let mut vs = MortarVariableState::default();

        let result = process_line_group(&group, &functions, &func_decls, &mut vs);
        assert_eq!(result, None, "All conditions false → None");
    }

    #[test]
    fn test_process_line_group_mixed_conditions() {
        let group = vec![
            make_line("Always shown"),
            make_conditional_line("True line", true_condition()),
            make_conditional_line("False line", false_condition()),
        ];
        let functions = MortarFunctionRegistry::new();
        let func_decls = vec![];
        let mut vs = MortarVariableState::default();
        vs.set("truthy_var", crate::MortarVariableValue::Boolean(true));

        let result = process_line_group(&group, &functions, &func_decls, &mut vs);
        assert_eq!(
            result,
            Some("Always shown\nTrue line".to_string()),
            "False line should be excluded"
        );
    }

    #[test]
    fn test_process_line_group_empty_lines_skipped() {
        let group = vec![make_line(""), make_line("Non-empty")];
        let functions = MortarFunctionRegistry::new();
        let func_decls = vec![];
        let mut vs = MortarVariableState::default();

        let result = process_line_group(&group, &functions, &func_decls, &mut vs);
        assert_eq!(
            result,
            Some("Non-empty".to_string()),
            "Empty lines should be excluded from join"
        );
    }
}
