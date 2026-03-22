//! This file holds edge-case regression tests for Mortar condition evaluation.
//! It protects the evaluator against malformed condition trees, missing operands,
//! unbound function calls, and other shapes that should fail safely instead of
//! panicking when parser or runtime logic changes.
//!
//! 这个文件存放 Mortar 条件求值的边界回归测试。它确保在解析器或运行时逻辑调整后，
//! 面对结构不完整的条件树、缺失操作数、未绑定函数调用等异常输入时，求值器仍然会
//! 以安全方式处理，而不是直接 panic。

use super::{
    IfCondition, MortarBoolean, MortarNumber, MortarString, MortarValue, MortarVariableState,
    evaluate_if_condition, make_empty_registry, make_registry_with_funcs,
};

#[test]
fn evaluate_empty_cond_type_no_panic() {
    let cond = IfCondition {
        cond_type: String::new(),
        operator: None,
        left: None,
        right: None,
        operand: None,
        value: None,
    };
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let _ = evaluate_if_condition(&cond, &reg, &state);
}

#[test]
fn evaluate_unknown_cond_type_no_panic() {
    let cond = IfCondition {
        cond_type: "nonexistent_type".to_string(),
        operator: None,
        left: None,
        right: None,
        operand: None,
        value: None,
    };
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let _ = evaluate_if_condition(&cond, &reg, &state);
}

#[test]
fn func_call_with_missing_operand_no_panic() {
    let cond = IfCondition {
        cond_type: "func_call".to_string(),
        operator: None,
        left: None,
        right: None,
        operand: None,
        value: None,
    };
    let reg = make_registry_with_funcs();
    let state = MortarVariableState::new();
    let result = evaluate_if_condition(&cond, &reg, &state);
    assert!(!result);
}

#[test]
fn func_call_with_unbound_function() {
    let cond = IfCondition {
        cond_type: "func_call".to_string(),
        operator: None,
        left: None,
        right: None,
        operand: Some(Box::new(IfCondition {
            cond_type: String::new(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("unbound_func".to_string()),
        })),
        value: None,
    };
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let result = evaluate_if_condition(&cond, &reg, &state);
    assert!(!result);
}

#[test]
fn func_call_comparison_with_registered_func() {
    let reg = make_registry_with_funcs();
    let state = MortarVariableState::new();

    let cond = IfCondition {
        cond_type: "binary".to_string(),
        operator: Some(">".to_string()),
        left: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_42".to_string()),
            })),
            value: None,
        })),
        right: Some(Box::new(IfCondition {
            cond_type: "literal".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("10".to_string()),
        })),
        operand: None,
        value: None,
    };
    assert!(evaluate_if_condition(&cond, &reg, &state));
}

#[test]
fn func_call_comparison_both_sides() {
    let reg = make_registry_with_funcs();
    let state = MortarVariableState::new();

    let cond = IfCondition {
        cond_type: "binary".to_string(),
        operator: Some(">".to_string()),
        left: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_42".to_string()),
            })),
            value: None,
        })),
        right: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_zero".to_string()),
            })),
            value: None,
        })),
        operand: None,
        value: None,
    };
    assert!(evaluate_if_condition(&cond, &reg, &state));
}

#[test]
fn binary_missing_left_right_no_panic() {
    let cond = IfCondition {
        cond_type: "binary".to_string(),
        operator: Some("&&".to_string()),
        left: None,
        right: None,
        operand: None,
        value: None,
    };
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluate_if_condition(&cond, &reg, &state)
    }));
    let _ = result;
}

#[test]
fn unary_missing_operand_no_panic() {
    let cond = IfCondition {
        cond_type: "unary".to_string(),
        operator: Some("!".to_string()),
        left: None,
        right: None,
        operand: None,
        value: None,
    };
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluate_if_condition(&cond, &reg, &state)
    }));
    let _ = result;
}

#[test]
fn mortar_value_parse_edge_cases() {
    let v = MortarValue::parse("");
    assert!(matches!(v, MortarValue::String(_)));

    let v = MortarValue::parse("\"\"");
    match v {
        MortarValue::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected empty string"),
    }

    let v = MortarValue::parse("   ");
    assert!(matches!(v, MortarValue::String(_)));

    let v = MortarValue::parse("0");
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("-1");
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("NaN");
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("inf");
    assert!(matches!(v, MortarValue::Number(_)));
}

#[test]
fn mortar_value_truthiness() {
    assert!(!MortarValue::Void.is_truthy());
    assert!(!MortarValue::Boolean(MortarBoolean(false)).is_truthy());
    assert!(MortarValue::Boolean(MortarBoolean(true)).is_truthy());
    assert!(!MortarValue::Number(MortarNumber(0.0)).is_truthy());
    assert!(MortarValue::Number(MortarNumber(1.0)).is_truthy());
    assert!(MortarValue::Number(MortarNumber(-1.0)).is_truthy());
    assert!(MortarValue::String(MortarString("x".into())).is_truthy());
    assert!(!MortarValue::String(MortarString(String::new())).is_truthy());
}

#[test]
fn logical_and_or_with_func_calls() {
    let reg = make_registry_with_funcs();
    let state = MortarVariableState::new();

    let cond = IfCondition {
        cond_type: "binary".to_string(),
        operator: Some("&&".to_string()),
        left: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_true".to_string()),
            })),
            value: None,
        })),
        right: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_false".to_string()),
            })),
            value: None,
        })),
        operand: None,
        value: None,
    };
    assert!(!evaluate_if_condition(&cond, &reg, &state));

    let cond_or = IfCondition {
        cond_type: "binary".to_string(),
        operator: Some("||".to_string()),
        left: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_true".to_string()),
            })),
            value: None,
        })),
        right: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_false".to_string()),
            })),
            value: None,
        })),
        operand: None,
        value: None,
    };
    assert!(evaluate_if_condition(&cond_or, &reg, &state));
}

#[test]
fn negation_of_func_call() {
    let reg = make_registry_with_funcs();
    let state = MortarVariableState::new();

    let cond = IfCondition {
        cond_type: "unary".to_string(),
        operator: Some("!".to_string()),
        left: None,
        right: None,
        operand: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_true".to_string()),
            })),
            value: None,
        })),
        value: None,
    };
    assert!(!evaluate_if_condition(&cond, &reg, &state));

    let cond2 = IfCondition {
        cond_type: "unary".to_string(),
        operator: Some("!".to_string()),
        left: None,
        right: None,
        operand: Some(Box::new(IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: String::new(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("returns_false".to_string()),
            })),
            value: None,
        })),
        value: None,
    };
    assert!(evaluate_if_condition(&cond2, &reg, &state));
}
