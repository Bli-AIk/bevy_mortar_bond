use crate::{MortarFunctionRegistry, MortarVariableState, evaluate_if_condition};

/// Cached result of a condition evaluation, used to ensure if/else mutual exclusivity.
///
/// 条件求值的缓存结果，用于确保 if/else 互斥。
#[derive(Default)]
pub struct CachedCondition {
    json: String,
    result: bool,
}

/// Evaluates a condition with caching to guarantee if/else mutual exclusivity.
///
/// 带缓存的条件求值，保证 if/else 互斥。
pub fn evaluate_condition_cached(
    condition: &mortar_compiler::IfCondition,
    functions: &MortarFunctionRegistry,
    variable_state: &MortarVariableState,
    cached: &mut Option<CachedCondition>,
) -> bool {
    let is_unary_not = condition.cond_type == "unary" && condition.operator.as_deref() == Some("!");

    if is_unary_not && let Some(result) = try_cache_negated(condition, cached) {
        return result;
    }

    if let Some(result) = try_cache_same(condition, cached) {
        return result;
    }

    let result = evaluate_if_condition(condition, functions, variable_state);

    if !is_unary_not && let Ok(json) = serde_json::to_string(condition) {
        *cached = Some(CachedCondition { json, result });
    }

    result
}

fn try_cache_negated(
    condition: &mortar_compiler::IfCondition,
    cached: &Option<CachedCondition>,
) -> Option<bool> {
    let operand = condition.operand.as_ref()?;
    let cache = cached.as_ref()?;
    let operand_json = serde_json::to_string(operand.as_ref()).ok()?;
    if operand_json != cache.json {
        return None;
    }
    let result = !cache.result;
    dev_info!(
        "Condition cache hit (negated): cached={} → result={}",
        cache.result,
        result
    );
    Some(result)
}

fn try_cache_same(
    condition: &mortar_compiler::IfCondition,
    cached: &Option<CachedCondition>,
) -> Option<bool> {
    let cache = cached.as_ref()?;
    let cond_json = serde_json::to_string(condition).ok()?;
    if cond_json != cache.json {
        return None;
    }
    dev_info!("Condition cache hit (same): result={}", cache.result);
    Some(cache.result)
}
