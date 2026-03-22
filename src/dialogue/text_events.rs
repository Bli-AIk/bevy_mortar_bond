//! # text_events.rs
//!
//! # text_events.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file derives runtime text-event indices from Mortar dialogue content. It remaps authored
//! event positions through interpolation and variable expansion so effects bound to dialogue text
//! still fire at the correct visible character index.
//!
//! 这个文件负责从 Mortar 对话内容里推导运行时文本事件索引。它会把作者写下的事件位置经过插值
//! 和变量展开重新映射，确保绑定到对话文本上的效果仍然在正确的可见字符位置触发。

use std::collections::HashMap;

use crate::{MortarVariableState, MortarVariableValue, TextData};

fn build_interpolation_index_map(
    parts: &[mortar_compiler::StringPart],
    variable_state: &MortarVariableState,
    asset_data: &mortar_compiler::MortaredData,
    all_events: &mut Vec<mortar_compiler::Event>,
) -> HashMap<usize, f64> {
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
            continue;
        }
        if part.part_type != "placeholder" {
            continue;
        }
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

    index_map.insert(original_pos as usize, rendered_pos);
    index_map
}

fn adjust_events_with_index_map(
    text_events: &[mortar_compiler::Event],
    index_map: &HashMap<usize, f64>,
    variable_state: &MortarVariableState,
    all_events: &mut Vec<mortar_compiler::Event>,
) {
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

pub fn collect_text_events(
    text_data: &TextData,
    variable_state: &MortarVariableState,
    asset_data: Option<&mortar_compiler::MortaredData>,
    current_text_content_idx: Option<usize>,
    node_data: &mortar_compiler::Node,
) -> Vec<mortar_compiler::Event> {
    let mut all_events = Vec::new();

    if let Some(parts) = &text_data.interpolated_parts {
        if let Some(asset_data) = asset_data {
            let index_map =
                build_interpolation_index_map(parts, variable_state, asset_data, &mut all_events);

            if let Some(text_events) = &text_data.events {
                adjust_events_with_index_map(
                    text_events,
                    &index_map,
                    variable_state,
                    &mut all_events,
                );
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
                .and_then(|v| match v {
                    MortarVariableValue::Number(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or(0.0)
        } else {
            index_override.value.parse::<f64>().unwrap_or(0.0)
        };

        if let Some(event_def) = asset_data
            .events
            .iter()
            .find(|event| event.name == event_name)
        {
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
