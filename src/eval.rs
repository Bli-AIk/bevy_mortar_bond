use bevy::prelude::*;

use crate::binder::{MortarBoolean, MortarFunctionRegistry, MortarNumber, MortarString};
use crate::variable_state::{MortarVariableState, MortarVariableValue};
use crate::{MortarValue, TextData};

/// Gets default return value based on type.
///
/// 根据类型获取默认返回值。
fn get_default_return_value(return_type: &str) -> String {
    match return_type {
        "Boolean" | "Bool" => "false".to_string(),
        "Number" => "0".to_string(),
        "String" => String::new(),
        _ => String::new(), // void or unknown
    }
}

/// Evaluates an IfCondition with support for function calls.
///
/// 评估 IfCondition，支持函数调用。
pub fn evaluate_if_condition(
    condition: &mortar_compiler::IfCondition,
    functions: &MortarFunctionRegistry,
    variable_state: &MortarVariableState,
) -> bool {
    match condition.cond_type.as_str() {
        "func_call" => {
            let func_name = condition.operand.as_ref().and_then(|op| op.value.clone());
            let Some(func_name) = func_name else {
                warn!("Function call condition missing function_name");
                return false;
            };
            let args: Vec<MortarValue> = condition
                .right
                .as_ref()
                .and_then(|r| r.value.as_ref())
                .map(|v| v.split_whitespace().map(MortarValue::parse).collect())
                .unwrap_or_default();

            if let Some(value) = functions.call(&func_name, &args) {
                value.is_truthy()
            } else {
                warn!(
                    "Condition function '{}' not bound, defaulting to false",
                    func_name
                );
                false
            }
        }
        "binary" => {
            let left = condition.left.as_ref().unwrap();
            let right = condition.right.as_ref().unwrap();

            match condition.operator.as_deref() {
                Some("&&") => {
                    evaluate_if_condition(left, functions, variable_state)
                        && evaluate_if_condition(right, functions, variable_state)
                }
                Some("||") => {
                    evaluate_if_condition(left, functions, variable_state)
                        || evaluate_if_condition(right, functions, variable_state)
                }
                _ => {
                    // For comparison operators, check if either operand is a func_call.
                    // variable_state alone cannot resolve func_calls, so we handle them here.
                    //
                    // 对比较运算符，检查是否有 func_call 操作数。
                    // variable_state 无法解析 func_call，需在此处处理。
                    if left.cond_type == "func_call" || right.cond_type == "func_call" {
                        let left_val = resolve_condition_value(left, functions, variable_state);
                        let right_val = resolve_condition_value(right, functions, variable_state);
                        compare_mortar_values(&left_val, &right_val, condition.operator.as_deref())
                    } else {
                        variable_state.evaluate_condition(condition)
                    }
                }
            }
        }
        "unary" => {
            let operand_result = evaluate_if_condition(
                condition.operand.as_ref().unwrap().as_ref(),
                functions,
                variable_state,
            );
            match condition.operator.as_deref() {
                Some("!") => !operand_result,
                _ => {
                    warn!("Unknown unary operator: {:?}", condition.operator);
                    false
                }
            }
        }
        _ => {
            // For other types, use variable_state's evaluation.
            //
            // 对其他类型使用 variable_state 的求值逻辑。
            variable_state.evaluate_condition(condition)
        }
    }
}

/// Resolves an if-condition operand to a concrete MortarValue.
/// Used when comparison operands include func_call types.
///
/// 将 if 条件操作数解析为具体的 MortarValue。
/// 用于比较操作数包含 func_call 类型的情况。
fn resolve_condition_value(
    cond: &mortar_compiler::IfCondition,
    functions: &MortarFunctionRegistry,
    variable_state: &MortarVariableState,
) -> MortarValue {
    match cond.cond_type.as_str() {
        "func_call" => {
            let func_name = cond.operand.as_ref().and_then(|op| op.value.clone());
            let args: Vec<MortarValue> = cond
                .right
                .as_ref()
                .and_then(|r| r.value.as_ref())
                .map(|v| v.split_whitespace().map(MortarValue::parse).collect())
                .unwrap_or_default();
            func_name
                .and_then(|name| functions.call(&name, &args))
                .unwrap_or(MortarValue::Void)
        }
        "identifier" => {
            let name = cond.value.as_deref().unwrap_or("");
            // Try parsing as number literal first (serializer outputs numbers as identifiers).
            //
            // 优先尝试解析为数值字面量（序列化器将数字输出为 identifier）。
            if let Ok(n) = name.parse::<f64>() {
                return MortarValue::Number(MortarNumber(n));
            }
            match variable_state.get(name) {
                Some(MortarVariableValue::Number(n)) => MortarValue::Number(MortarNumber(*n)),
                Some(MortarVariableValue::String(s)) => {
                    MortarValue::String(MortarString(s.clone()))
                }
                Some(MortarVariableValue::Boolean(b)) => MortarValue::Boolean(MortarBoolean(*b)),
                None => MortarValue::Void,
            }
        }
        "literal" => {
            let val = cond.value.as_deref().unwrap_or("0");
            MortarValue::parse(val)
        }
        "enum_member" => {
            let val = cond.value.as_deref().unwrap_or("");
            MortarValue::String(MortarString(val.to_string()))
        }
        _ => MortarValue::Void,
    }
}

/// Compares two MortarValues with a given operator.
///
/// 使用给定运算符比较两个 MortarValue。
fn compare_mortar_values(left: &MortarValue, right: &MortarValue, op: Option<&str>) -> bool {
    match op {
        Some("==") => match (left, right) {
            (MortarValue::Number(l), MortarValue::Number(r)) => {
                (l.as_f64() - r.as_f64()).abs() < f64::EPSILON
            }
            (MortarValue::String(l), MortarValue::String(r)) => l.as_str() == r.as_str(),
            (MortarValue::Boolean(l), MortarValue::Boolean(r)) => l.as_bool() == r.as_bool(),
            _ => false,
        },
        Some("!=") => !compare_mortar_values(left, right, Some("==")),
        Some(">") => compare_numbers(left, right, |a, b| a > b),
        Some("<") => compare_numbers(left, right, |a, b| a < b),
        Some(">=") => compare_numbers(left, right, |a, b| a >= b),
        Some("<=") => compare_numbers(left, right, |a, b| a <= b),
        _ => false,
    }
}

fn compare_numbers(left: &MortarValue, right: &MortarValue, cmp: fn(f64, f64) -> bool) -> bool {
    match (left.as_number(), right.as_number()) {
        (Some(l), Some(r)) => cmp(l.as_f64(), r.as_f64()),
        _ => false,
    }
}

/// Evaluates a condition by calling the bound function.
///
/// 通过调用绑定函数来评估条件。
pub fn evaluate_condition(
    condition: &mortar_compiler::Condition,
    functions: &MortarFunctionRegistry,
    _function_decls: &[mortar_compiler::Function],
) -> bool {
    // Parse arguments.
    //
    // 解析参数。
    let args: Vec<MortarValue> = condition
        .args
        .iter()
        .map(|arg| MortarValue::parse(arg))
        .collect();

    // Call the function.
    //
    // 调用函数。
    if let Some(value) = functions.call(&condition.condition_type, &args) {
        value.is_truthy()
    } else {
        // Function not found - default to false.
        //
        // 未找到函数时默认返回 false。
        warn!(
            "Condition function '{}' not bound, defaulting to false",
            condition.condition_type
        );
        false
    }
}

/// Processes interpolated text by calling bound functions and resolving variables.
///
/// 通过调用绑定函数和解析变量来处理插值文本。
pub fn process_interpolated_text(
    text_data: &TextData,
    functions: &MortarFunctionRegistry,
    function_decls: &[mortar_compiler::Function],
    variable_state: &MortarVariableState,
) -> String {
    // If there are no interpolated parts, return the original text.
    //
    // 如果没有插值片段，则直接返回原始文本。
    let Some(parts) = &text_data.interpolated_parts else {
        return text_data.value.clone();
    };

    let mut result = String::new();
    for part in parts {
        match part.part_type.as_str() {
            "text" => {
                result.push_str(&part.content);
            }
            "expression" => {
                let Some(func_name) = &part.function_name else {
                    result.push_str(&part.content);
                    continue;
                };
                let args: Vec<MortarValue> = part
                    .args
                    .iter()
                    .map(|arg| MortarValue::parse(arg))
                    .collect();

                if let Some(value) = functions.call(func_name, &args) {
                    result.push_str(&value.to_display_string());
                } else {
                    let return_type = function_decls
                        .iter()
                        .find(|f| f.name == *func_name)
                        .and_then(|f| f.return_type.as_deref())
                        .unwrap_or("void");

                    let default_value = get_default_return_value(return_type);
                    warn!(
                        "Function '{}' not bound, using default return value: {}",
                        func_name, default_value
                    );
                    result.push_str(&default_value);
                }
            }
            "placeholder" => {
                // Extract variable name from placeholder (e.g., "{status}" -> "status").
                //
                // 从占位符中提取变量名（如 "{status}" -> "status"）。
                let var_name = part.content.trim_matches(|c| c == '{' || c == '}');

                // First try to get as a regular variable.
                //
                // 优先尝试作为普通变量获取。
                if let Some(value) = variable_state.get(var_name) {
                    result.push_str(&value.to_display_string());
                } else if let Some(branch_text) = variable_state.get_branch_text(var_name) {
                    // Try to get as a branch variable.
                    //
                    // 尝试作为分支变量获取。
                    result.push_str(&branch_text);
                } else {
                    // Variable not found, keep placeholder.
                    //
                    // 未找到变量时保留占位符。
                    warn!("Variable '{}' not found, keeping placeholder", var_name);
                    result.push_str(&part.content);
                }
            }
            _ => {
                // Unknown type, keep the content.
                //
                // 未知类型则保留原内容。
                result.push_str(&part.content);
            }
        }
    }

    result
}
