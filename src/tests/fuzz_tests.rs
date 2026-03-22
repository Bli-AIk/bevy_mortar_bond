//! This file defines the shared fuzzing strategies and helper registries used by
//! Mortar's property-style tests. It is the support module behind the fuzz test
//! suite, generating random conditions, literals, and lightweight registries that
//! let individual test files focus on one runtime guarantee at a time.
//!
//! 这个文件定义了 Mortar 属性测试共用的 fuzz 策略和辅助注册表。它是整组 fuzz
//! 测试的支撑模块，负责生成随机条件、字面量和轻量级注册表，让各个测试文件可以把
//! 注意力集中在单一运行时保证上。

use crate::binder::{MortarBoolean, MortarFunctionRegistry, MortarNumber, MortarString};
use crate::variable_state::MortarVariableState;
use crate::{MortarValue, evaluate_if_condition};
use mortar_compiler::IfCondition;
use proptest::prelude::*;

// --- Strategy helpers ---

fn arb_leaf_condition() -> impl Strategy<Value = IfCondition> {
    prop_oneof![
        // identifier
        "[a-z_]{1,10}".prop_map(|name| IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some(name),
        }),
        // literal
        prop_oneof![
            (0i64..1000).prop_map(|n| n.to_string()),
            Just("true".to_string()),
            Just("false".to_string()),
            "[a-z]{1,10}".prop_map(|s| format!("\"{}\"", s)),
        ]
        .prop_map(|val| IfCondition {
            cond_type: "literal".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some(val),
        }),
        // func_call
        "[a-z_]{1,10}".prop_map(|name| IfCondition {
            cond_type: "func_call".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: Some(Box::new(IfCondition {
                cond_type: "".to_string(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some(name),
            })),
            value: None,
        }),
    ]
}

fn arb_if_condition() -> impl Strategy<Value = IfCondition> {
    arb_leaf_condition().prop_recursive(3, 16, 3, |inner| {
        prop_oneof![
            // binary comparison
            (
                inner.clone(),
                inner.clone(),
                prop_oneof![
                    Just("==".to_string()),
                    Just("!=".to_string()),
                    Just(">".to_string()),
                    Just("<".to_string()),
                    Just(">=".to_string()),
                    Just("<=".to_string()),
                ]
            )
                .prop_map(|(left, right, op)| IfCondition {
                    cond_type: "binary".to_string(),
                    operator: Some(op),
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                    operand: None,
                    value: None,
                }),
            // logical and/or
            (
                inner.clone(),
                inner.clone(),
                prop_oneof![Just("&&".to_string()), Just("||".to_string()),]
            )
                .prop_map(|(left, right, op)| IfCondition {
                    cond_type: "binary".to_string(),
                    operator: Some(op),
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                    operand: None,
                    value: None,
                }),
            // unary negation
            inner.clone().prop_map(|operand| IfCondition {
                cond_type: "unary".to_string(),
                operator: Some("!".to_string()),
                left: None,
                right: None,
                operand: Some(Box::new(operand)),
                value: None,
            }),
        ]
    })
}

fn arb_mortar_value() -> impl Strategy<Value = MortarValue> {
    prop_oneof![
        any::<f64>()
            .prop_filter("finite", |v| v.is_finite())
            .prop_map(|n| MortarValue::Number(MortarNumber(n))),
        "[a-zA-Z0-9_ ]{0,20}".prop_map(|s| MortarValue::String(MortarString(s))),
        any::<bool>().prop_map(|b| MortarValue::Boolean(MortarBoolean(b))),
        Just(MortarValue::Void),
    ]
}

fn make_empty_registry() -> MortarFunctionRegistry {
    MortarFunctionRegistry::new()
}

fn make_registry_with_funcs() -> MortarFunctionRegistry {
    let mut reg = MortarFunctionRegistry::new();
    reg.register("returns_true", |_| {
        MortarValue::Boolean(MortarBoolean(true))
    });
    reg.register("returns_false", |_| {
        MortarValue::Boolean(MortarBoolean(false))
    });
    reg.register("returns_42", |_| MortarValue::Number(MortarNumber(42.0)));
    reg.register("returns_zero", |_| MortarValue::Number(MortarNumber(0.0)));
    reg.register("returns_hello", |_| {
        MortarValue::String(MortarString("hello".into()))
    });
    reg.register("identity", |args| {
        args.first().cloned().unwrap_or(MortarValue::Void)
    });
    reg
}

// --- Property tests: evaluate_if_condition ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// evaluate_if_condition must never panic on arbitrary condition trees.
    #[test]
    fn evaluate_never_panics(cond in arb_if_condition()) {
        let reg = make_empty_registry();
        let state = MortarVariableState::new();
        let _ = evaluate_if_condition(&cond, &reg, &state);
    }

    /// evaluate_if_condition with registered functions must not panic.
    #[test]
    fn evaluate_with_funcs_never_panics(cond in arb_if_condition()) {
        let reg = make_registry_with_funcs();
        let state = MortarVariableState::new();
        let _ = evaluate_if_condition(&cond, &reg, &state);
    }

    /// MortarValue::parse must never panic on arbitrary strings.
    #[test]
    fn mortar_value_parse_never_panics(input in "\\PC{0,100}") {
        let _ = MortarValue::parse(&input);
    }

    /// MortarValue::is_truthy must never panic.
    #[test]
    fn mortar_value_is_truthy_never_panics(val in arb_mortar_value()) {
        let _ = val.is_truthy();
    }

    /// MortarValue::to_display_string must never panic.
    #[test]
    fn mortar_value_display_never_panics(val in arb_mortar_value()) {
        let _ = val.to_display_string();
    }

    /// Variable state set/get roundtrip consistency.
    #[test]
    fn variable_state_set_get_roundtrip(
        name in "[a-z_]{1,15}",
        val_f64 in any::<f64>().prop_filter("finite", |v| v.is_finite()),
    ) {
        use crate::variable_state::MortarVariableValue;
        let mut state = MortarVariableState::new();
        state.set(&name, MortarVariableValue::Number(val_f64));
        let got = state.get(&name);
        match got {
            Some(MortarVariableValue::Number(n)) => prop_assert_eq!(*n, val_f64),
            other => prop_assert!(false, "Expected Number, got {:?}", other),
        }
    }

    /// Multiple set/get operations maintain consistency.
    #[test]
    fn variable_state_multi_set(
        ops in prop::collection::vec(
            ("[a-z]{1,5}", prop_oneof![
                any::<f64>().prop_filter("finite", |v| v.is_finite()).prop_map(|n| MortarValue::Number(MortarNumber(n))),
                "[a-z]{1,10}".prop_map(|s| MortarValue::String(MortarString(s))),
                any::<bool>().prop_map(|b| MortarValue::Boolean(MortarBoolean(b))),
            ]),
            1..20,
        ),
    ) {
        use crate::variable_state::MortarVariableValue;
        let mut state = MortarVariableState::new();
        let mut expected: std::collections::HashMap<String, MortarValue> = std::collections::HashMap::new();

        for (name, val) in &ops {
            let var_val = match val {
                MortarValue::Number(n) => MortarVariableValue::Number(n.as_f64()),
                MortarValue::String(s) => MortarVariableValue::String(s.as_str().to_string()),
                MortarValue::Boolean(b) => MortarVariableValue::Boolean(b.as_bool()),
                MortarValue::Void => continue,
            };
            state.set(name, var_val);
            expected.insert(name.clone(), val.clone());
        }

        for name in expected.keys() {
            let got = state.get(name);
            prop_assert!(got.is_some(), "Missing key {}", name);
        }
    }
}

mod cache_tests;
mod edge_case_tests;
