//! This file covers Mortar's basic value and variable-state primitives.
//! It verifies parsing, display formatting, conversions, and variable mutation so
//! higher-level dialogue tests can rely on these low-level building blocks.
//!
//! 这个文件覆盖 Mortar 最基础的值类型和变量状态类型。它验证解析、显示格式、类型转换
//! 和变量修改语义，让更高层的对话测试能够建立在可靠的底层基元之上。

use super::*;

#[test]
fn test_mortar_value_parse_string() {
    let val = MortarValue::parse("\"hello\"");
    match val {
        MortarValue::String(s) => assert_eq!(s.0, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_mortar_value_parse_number() {
    let val = MortarValue::parse("42.5");
    match val {
        MortarValue::Number(n) => assert_eq!(n.0, 42.5),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_mortar_value_parse_boolean() {
    let val_true = MortarValue::parse("true");
    match val_true {
        MortarValue::Boolean(b) => assert!(b.0),
        _ => panic!("Expected Boolean"),
    }

    let val_false = MortarValue::parse("false");
    match val_false {
        MortarValue::Boolean(b) => assert!(!b.0),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_mortar_value_to_display_string() {
    assert_eq!(
        MortarValue::String(MortarString("test".to_string())).to_display_string(),
        "test"
    );
    assert_eq!(
        MortarValue::Number(MortarNumber(3.42)).to_display_string(),
        "3.42"
    );
    assert_eq!(
        MortarValue::Boolean(MortarBoolean(true)).to_display_string(),
        "true"
    );
}

#[test]
fn test_variable_state_from_variables() {
    let variables = vec![
        mortar_compiler::Variable {
            name: "health".to_string(),
            var_type: "Number".to_string(),
            value: Some(serde_json::json!(100.0)),
        },
        mortar_compiler::Variable {
            name: "player_name".to_string(),
            var_type: "String".to_string(),
            value: Some(serde_json::json!("Alice")),
        },
    ];

    let state = MortarVariableState::from_variables(&variables, &[], &[]);

    assert_eq!(
        state.get("health"),
        Some(&MortarVariableValue::Number(100.0))
    );
    assert_eq!(
        state.get("player_name"),
        Some(&MortarVariableValue::String("Alice".to_string()))
    );
}

#[test]
fn test_variable_state_execute_assignment() {
    let variables = vec![mortar_compiler::Variable {
        name: "count".to_string(),
        var_type: "Number".to_string(),
        value: Some(serde_json::json!(0.0)),
    }];

    let mut state = MortarVariableState::from_variables(&variables, &[], &[]);

    state.execute_assignment("count", "42");
    assert_eq!(state.get("count"), Some(&MortarVariableValue::Number(42.0)));

    state.execute_assignment("count", "\"text\"");
    assert_eq!(
        state.get("count"),
        Some(&MortarVariableValue::String("\"text\"".to_string()))
    );
}

#[test]
fn test_variable_state_evaluate_condition() {
    let variables = vec![
        mortar_compiler::Variable {
            name: "has_key".to_string(),
            var_type: "Boolean".to_string(),
            value: Some(serde_json::json!(true)),
        },
        mortar_compiler::Variable {
            name: "is_locked".to_string(),
            var_type: "Boolean".to_string(),
            value: Some(serde_json::json!(false)),
        },
    ];

    let state = MortarVariableState::from_variables(&variables, &[], &[]);

    let condition = mortar_compiler::IfCondition {
        cond_type: "identifier".to_string(),
        operator: None,
        left: None,
        right: None,
        operand: None,
        value: Some("has_key".to_string()),
    };
    assert!(state.evaluate_condition(&condition));

    let condition = mortar_compiler::IfCondition {
        cond_type: "identifier".to_string(),
        operator: None,
        left: None,
        right: None,
        operand: None,
        value: Some("is_locked".to_string()),
    };
    assert!(!state.evaluate_condition(&condition));
}

#[test]
fn test_variable_value_to_display_string() {
    assert_eq!(
        MortarVariableValue::Number(3.42).to_display_string(),
        "3.42"
    );
    assert_eq!(
        MortarVariableValue::String("test".to_string()).to_display_string(),
        "test"
    );
    assert_eq!(
        MortarVariableValue::Boolean(true).to_display_string(),
        "true"
    );
}

#[test]
fn test_variable_state_branch_text() {
    let mut state = MortarVariableState::default();

    state.set_branch_text("branch1".to_string(), "Option A".to_string());
    state.set_branch_text("branch2".to_string(), "Option B".to_string());

    assert_eq!(
        state.get_branch_text("branch1"),
        Some("Option A".to_string())
    );
    assert_eq!(
        state.get_branch_text("branch2"),
        Some("Option B".to_string())
    );
    assert_eq!(state.get_branch_text("nonexistent"), None);
}

#[test]
fn test_variable_state_parse_value() {
    let variables = vec![mortar_compiler::Variable {
        name: "test".to_string(),
        var_type: "Number".to_string(),
        value: Some(serde_json::json!(42.0)),
    }];

    let mut state = MortarVariableState::from_variables(&variables, &[], &[]);

    state.execute_assignment("test", "100");
    assert_eq!(state.get("test"), Some(&MortarVariableValue::Number(100.0)));

    state.execute_assignment("test", "true");
    assert_eq!(state.get("test"), Some(&MortarVariableValue::Boolean(true)));

    state.execute_assignment("test", "false");
    assert_eq!(
        state.get("test"),
        Some(&MortarVariableValue::Boolean(false))
    );
}

#[test]
fn test_mortar_value_parse_edge_cases() {
    let val = MortarValue::parse("");
    match val {
        MortarValue::String(s) => assert_eq!(s.0, ""),
        _ => panic!("Expected String"),
    }

    let val = MortarValue::parse("\"\"");
    match val {
        MortarValue::String(s) => assert_eq!(s.0, ""),
        _ => panic!("Expected String"),
    }

    let val = MortarValue::parse("-42.5");
    match val {
        MortarValue::Number(n) => assert_eq!(n.0, -42.5),
        _ => panic!("Expected Number"),
    }

    let val = MortarValue::parse("0");
    match val {
        MortarValue::Number(n) => assert_eq!(n.0, 0.0),
        _ => panic!("Expected Number"),
    }
}
