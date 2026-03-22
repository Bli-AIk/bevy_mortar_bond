//! This file contains cache-oriented property tests for Mortar condition evaluation.
//! It checks that cached branches keep their mutual-exclusion behavior and that
//! repeated evaluations do not quietly change semantics when function calls or
//! binary comparisons are involved.
//!
//! 这个文件放的是面向缓存行为的 Mortar 条件属性测试。它验证带缓存的分支仍然保持
//! 互斥语义，并确保在涉及函数调用和二元比较时，重复求值不会因为缓存机制而悄悄改变结果。

use super::{
    MortarFunctionRegistry, MortarNumber, MortarValue, MortarVariableState, arb_if_condition,
    evaluate_if_condition, make_empty_registry,
};
use mortar_compiler::IfCondition;
use proptest::prelude::*;

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

    let r0 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r1 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r2 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    let r3 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);

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

    reg.register("get_hp", |_| MortarValue::Number(MortarNumber(20.0)));
    reg.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));

    let if_cond = make_binary_ge_func_call("get_hp", "get_max_hp");
    let else_cond = make_unary_not(if_cond.clone());

    let r1 = evaluate_condition_cached(&if_cond, &reg, &state, &mut cache);
    let r2 = evaluate_condition_cached(&else_cond, &reg, &state, &mut cache);
    assert!(r1 && !r2, "first dialogue: if=true, else=false");

    cache = None;

    let mut reg2 = MortarFunctionRegistry::new();
    reg2.register("get_hp", |_| MortarValue::Number(MortarNumber(10.0)));
    reg2.register("get_max_hp", |_| MortarValue::Number(MortarNumber(20.0)));

    let r3 = evaluate_condition_cached(&if_cond, &reg2, &state, &mut cache);
    let r4 = evaluate_condition_cached(&else_cond, &reg2, &state, &mut cache);
    assert!(!r3 && r4, "second dialogue: if=false, else=true");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

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
