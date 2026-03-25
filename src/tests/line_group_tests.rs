//! Tests how `DialogueState` treats consecutive `line` entries.
//! It verifies the line-group behavior that the dialogue runtime relies on when
//! batching authored line fragments into a single navigable unit.
//!
//! 测试 `DialogueState` 如何处理连续出现的 `line` 条目。对话运行时依赖
//! 这套“行分组”语义，把作者写下的多段 line 片段收拢成一个可导航的整体；这里负责
//! 确认那条语义不会在后续重构里被破坏。

use crate::*;

fn create_line_group_node() -> mortar_compiler::Node {
    use serde_json::json;
    mortar_compiler::Node {
        name: "LineGroupNode".to_string(),
        content: vec![
            json!({ "type": "line", "value": "Line A" }),
            json!({ "type": "line", "value": "Line B" }),
            json!({ "type": "line", "value": "Line C" }),
        ],
        branches: None,
        variables: vec![],
        next: None,
    }
}

#[test]
fn test_line_group_is_line_flag() {
    let node = create_line_group_node();
    let state = DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

    let td = state.current_text_data().unwrap();
    assert!(td.is_line, "First entry should be is_line=true");
}

#[test]
fn test_line_group_end_covers_consecutive_lines() {
    let node = create_line_group_node();
    let state = DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

    let group = state.current_line_group().unwrap();
    assert_eq!(
        group.len(),
        3,
        "All 3 consecutive lines should form one group"
    );
    assert_eq!(group[0].value, "Line A");
    assert_eq!(group[1].value, "Line B");
    assert_eq!(group[2].value, "Line C");
}

#[test]
fn test_line_group_next_text_skips_entire_group() {
    let node = create_line_group_node();
    let mut state =
        DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

    assert!(
        !state.has_next_text(),
        "3-line group should be the only step"
    );
    assert!(
        !state.next_text(),
        "next_text should return false when no text after group"
    );
}

#[test]
fn test_mixed_text_and_line_group() {
    use serde_json::json;
    let node = mortar_compiler::Node {
        name: "MixedNode".to_string(),
        content: vec![
            json!({ "type": "text", "value": "Step 1" }),
            json!({ "type": "line", "value": "Line A" }),
            json!({ "type": "line", "value": "Line B" }),
            json!({ "type": "text", "value": "Step 2" }),
        ],
        branches: None,
        variables: vec![],
        next: None,
    };
    let mut state = DialogueState::new("test.mortar".to_string(), "MixedNode".to_string(), node);

    assert_eq!(state.current_text(), Some("Step 1"));
    assert!(!state.current_text_data().unwrap().is_line);
    assert!(state.has_next_text());

    assert!(state.next_text());

    let td = state.current_text_data().unwrap();
    assert!(td.is_line);
    let group = state.current_line_group().unwrap();
    assert_eq!(group.len(), 2);
    assert_eq!(group[0].value, "Line A");
    assert_eq!(group[1].value, "Line B");
    assert!(state.has_next_text());

    assert!(state.next_text());

    assert_eq!(state.current_text(), Some("Step 2"));
    assert!(!state.current_text_data().unwrap().is_line);
    assert!(!state.has_next_text());
}

#[test]
fn test_line_group_with_conditions() {
    use serde_json::json;
    let node = mortar_compiler::Node {
        name: "CondNode".to_string(),
        content: vec![
            json!({ "type": "line", "value": "Always shown" }),
            json!({
                "type": "line",
                "value": "Conditional line",
                "condition": {
                    "type": "identifier",
                    "value": "some_flag"
                }
            }),
        ],
        branches: None,
        variables: vec![],
        next: None,
    };
    let state = DialogueState::new("test.mortar".to_string(), "CondNode".to_string(), node);

    let group = state.current_line_group().unwrap();
    assert_eq!(
        group.len(),
        2,
        "Conditional line is still part of the group"
    );
    assert!(group[0].condition.is_none());
    assert!(group[1].condition.is_some());

    assert!(
        !state.has_next_text(),
        "Conditional line group should be one step"
    );
}

#[test]
fn test_line_group_last_content_index() {
    use serde_json::json;
    let node = mortar_compiler::Node {
        name: "RunAfterLine".to_string(),
        content: vec![
            json!({ "type": "line", "value": "Line A" }),
            json!({ "type": "line", "value": "Line B" }),
            json!({ "type": "line", "value": "Line C" }),
            json!({ "type": "run_event", "name": "sfx" }),
            json!({ "type": "text", "value": "After group" }),
        ],
        branches: None,
        variables: vec![],
        next: None,
    };
    let state = DialogueState::new("test.mortar".to_string(), "RunAfterLine".to_string(), node);

    assert_eq!(
        state.line_group_last_content_index(),
        Some(2),
        "Last line in group should have content index 2"
    );

    assert_eq!(
        state.current_text_content_index(),
        Some(0),
        "First line in group should have content index 0"
    );
}

#[test]
fn test_line_group_single_text_last_content_index() {
    use serde_json::json;
    let node = mortar_compiler::Node {
        name: "SingleText".to_string(),
        content: vec![json!({ "type": "text", "value": "Hello" })],
        branches: None,
        variables: vec![],
        next: None,
    };
    let state = DialogueState::new("test.mortar".to_string(), "SingleText".to_string(), node);

    assert_eq!(state.current_text_content_index(), Some(0));
    assert_eq!(state.line_group_last_content_index(), Some(0));
}
