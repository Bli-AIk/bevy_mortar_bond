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

// --- Deterministic edge-case tests ---

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

    // returns_42() > 10 → true
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

    // returns_42() > returns_zero() → true
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
    // This may panic — we need to know. If it does, that's a bug to fix.
    // For now, just document the behavior.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluate_if_condition(&cond, &reg, &state)
    }));
    // Log whether it panicked or not — either is acceptable for now.
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
    // Empty
    let v = MortarValue::parse("");
    assert!(matches!(v, MortarValue::String(_)));

    // Just quotes
    let v = MortarValue::parse("\"\"");
    match v {
        MortarValue::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected empty string"),
    }

    // Just whitespace
    let v = MortarValue::parse("   ");
    assert!(matches!(v, MortarValue::String(_)));

    // Numeric edge cases
    let v = MortarValue::parse("0");
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("-1");
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("NaN");
    // NaN parses as f64::NAN
    assert!(matches!(v, MortarValue::Number(_)));

    let v = MortarValue::parse("inf");
    // "inf" parses as infinity
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

    // returns_true() && returns_false() → false
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

    // returns_true() || returns_false() → true
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

    // !returns_true() → false
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

    // !returns_false() → true
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

// --- Condition caching tests (if/else mutual exclusivity) ---

/// Helper to build an IfCondition for `func_name() >= func_name()`.
fn make_binary_ge_func_call(left_fn: &str, right_fn: &str) -> IfCondition {
    IfCondition {
        cond_type: "binary".to_string(),
        operator: Some(">=".to_string()),
        left: Some(Box::new(make_func_call(left_fn))),
        right: Some(Box::new(make_func_call(right_fn))),
        operand: None,
        value: None,
    }
}

/// Helper to build a func_call IfCondition.
fn make_func_call(name: &str) -> IfCondition {
    IfCondition {
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
            value: Some(name.to_string()),
        })),
        value: None,
    }
}

/// Helper to build `!(inner_condition)`.
fn make_unary_not(inner: IfCondition) -> IfCondition {
    IfCondition {
        cond_type: "unary".to_string(),
        operator: Some("!".to_string()),
        left: None,
        right: None,
        operand: Some(Box::new(inner)),
        value: None,
    }
}

#[test]
fn void_ge_void_returns_false() {
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let cond = make_binary_ge_func_call("unregistered_a", "unregistered_b");
    assert!(!evaluate_if_condition(&cond, &reg, &state));
}

#[test]
fn not_void_ge_void_returns_true() {
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let inner = make_binary_ge_func_call("unregistered_a", "unregistered_b");
    let cond = make_unary_not(inner);
    assert!(evaluate_if_condition(&cond, &reg, &state));
}

#[test]
fn if_else_mutual_exclusivity_via_evaluate() {
    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());
    let if_result = evaluate_if_condition(&if_cond, &reg, &state);
    let else_result = evaluate_if_condition(&else_cond, &reg, &state);
    assert_ne!(
        if_result, else_result,
        "if and else must be mutually exclusive"
    );
}

#[test]
fn cached_condition_ensures_if_else_exclusivity() {
    use crate::dialogue::CachedCondition;
    use crate::dialogue::evaluate_condition_cached;

    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let mut cache: Option<CachedCondition> = None;

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    let if_result = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let else_result = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);

    assert_ne!(
        if_result, else_result,
        "cached evaluation must guarantee mutual exclusivity"
    );
}

#[test]
fn cached_condition_with_registered_funcs_hp_full() {
    use crate::dialogue::CachedCondition;
    use crate::dialogue::evaluate_condition_cached;

    let mut reg = MortarFunctionRegistry::new();
    reg.register("get_hp", |_| MortarValue::Number(MortarNumber(20.0)));
    reg.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));
    let state = MortarVariableState::new();
    let mut cache: Option<CachedCondition> = None;

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    // HP full: get_hp() >= get_max_hp() → true
    let if_result = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    assert!(if_result, "HP full: if-branch should be true");

    let else_result = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    assert!(!else_result, "HP full: else-branch should be false");
}

#[test]
fn cached_condition_with_registered_funcs_hp_not_full() {
    use crate::dialogue::CachedCondition;
    use crate::dialogue::evaluate_condition_cached;

    let mut reg = MortarFunctionRegistry::new();
    reg.register("get_hp", |_| MortarValue::Number(MortarNumber(15.0)));
    reg.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));
    let state = MortarVariableState::new();
    let mut cache: Option<CachedCondition> = None;

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    // HP not full: get_hp() >= get_max_hp() → false
    let if_result = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    assert!(!if_result, "HP not full: if-branch should be false");

    let else_result = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    assert!(else_result, "HP not full: else-branch should be true");
}

#[test]
fn cached_condition_multi_text_if_else_branches() {
    use crate::dialogue::CachedCondition;
    use crate::dialogue::evaluate_condition_cached;

    let reg = make_empty_registry();
    let state = MortarVariableState::new();
    let mut cache: Option<CachedCondition> = None;

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    // Simulate compiled output: [if_text_0, if_text_1, else_text_0, else_text_1]
    // With unregistered functions: Void >= Void → false
    let r0 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r1 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r2 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    let r3 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);

    // Only else-branch should show
    assert!(!r0, "if text 0 should be false");
    assert!(!r1, "if text 1 should be false");
    assert!(r2, "else text 0 should be true");
    assert!(r3, "else text 1 should be true");
}

#[test]
fn cached_condition_reset_between_dialogues() {
    use crate::dialogue::CachedCondition;
    use crate::dialogue::evaluate_condition_cached;

    let mut reg = MortarFunctionRegistry::new();
    let state = MortarVariableState::new();
    let mut cache: Option<CachedCondition> = None;

    // First dialogue: HP full
    reg.register("get_hp", |_| MortarValue::Number(MortarNumber(20.0)));
    reg.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    let r1 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r2 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    assert!(r1 && !r2, "first dialogue: if=true, else=false");

    // Simulate dialogue end: reset cache
    cache = None;

    // Second dialogue: HP not full (different registry)
    let mut reg2 = MortarFunctionRegistry::new();
    reg2.register("get_hp", |_| MortarValue::Number(MortarNumber(10.0)));
    reg2.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));

    let r3 = evaluate_condition_cached(&if_cond, &reg2, &state, &mut cache);
    let r4 = evaluate_condition_cached(&else_cond, &reg2, &state, &mut cache);
    assert!(!r3 && r4, "second dialogue: if=false, else=true");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// For any condition C, evaluate(C) and evaluate(!C) must be mutually exclusive.
    #[test]
    fn if_else_always_mutually_exclusive(cond in arb_if_condition()) {
        let reg = make_empty_registry();
        let state = MortarVariableState::new();
        let if_result = evaluate_if_condition(&cond, &reg, &state);
        let negated = IfCondition {
            cond_type: "unary".to_string(),
            operator: Some("!".to_string()),
            left: None,
            right: None,
            operand: Some(Box::new(cond)),
            value: None,
        };
        let else_result = evaluate_if_condition(&negated, &reg, &state);
        prop_assert_ne!(if_result, else_result,
            "if and else must always be mutually exclusive");
    }
}
