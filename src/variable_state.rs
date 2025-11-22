//! Variable state management for Mortar runtime.
//!
//! Mortar 运行时的变量状态管理。

use bevy::prelude::*;
use mortar_compiler::{IfCondition, Variable};
use std::collections::HashMap;

/// Runtime value for a Mortar variable.
///
/// Mortar 变量的运行时值。
#[derive(Debug, Clone, PartialEq)]
pub enum MortarVariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl MortarVariableValue {
    /// Parse a value from JSON.
    ///
    /// 从 JSON 解析值。
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(s) => Some(MortarVariableValue::String(s.clone())),
            serde_json::Value::Number(n) => n.as_f64().map(MortarVariableValue::Number),
            serde_json::Value::Bool(b) => Some(MortarVariableValue::Boolean(*b)),
            _ => None,
        }
    }

    /// Convert to display string.
    ///
    /// 转换为显示字符串。
    pub fn to_display_string(&self) -> String {
        match self {
            MortarVariableValue::String(s) => s.clone(),
            MortarVariableValue::Number(n) => n.to_string(),
            MortarVariableValue::Boolean(b) => b.to_string(),
        }
    }
}

/// Component that manages variable state for a Mortar dialogue runtime.
///
/// 管理 Mortar 对话运行时变量状态的组件。
#[derive(Component, Debug, Clone)]
pub struct MortarVariableState {
    variables: HashMap<String, MortarVariableValue>,
}

impl Default for MortarVariableState {
    fn default() -> Self {
        Self::new()
    }
}

impl MortarVariableState {
    /// Create a new empty variable state.
    ///
    /// 创建一个新的空变量状态。
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Initialize from a list of variable declarations.
    ///
    /// 从变量声明列表初始化。
    pub fn from_variables(variables: &[Variable]) -> Self {
        let mut state = Self::new();
        for var in variables {
            // Try to parse the value
            if let Some(value) = &var.value {
                if let Some(parsed_value) = MortarVariableValue::from_json(value) {
                    state.set(&var.name, parsed_value);
                }
            } else {
                // Set default value based on type
                let default_value = match var.var_type.as_str() {
                    "String" => MortarVariableValue::String(String::new()),
                    "Number" => MortarVariableValue::Number(0.0),
                    "Boolean" | "Bool" => MortarVariableValue::Boolean(false),
                    _ => continue, // Skip unknown types
                };
                state.set(&var.name, default_value);
            }
        }
        state
    }

    /// Set a variable value.
    ///
    /// 设置变量值。
    pub fn set(&mut self, name: &str, value: MortarVariableValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// Get a variable value.
    ///
    /// 获取变量值。
    pub fn get(&self, name: &str) -> Option<&MortarVariableValue> {
        self.variables.get(name)
    }

    /// Evaluate a condition.
    ///
    /// 评估条件。
    pub fn evaluate_condition(&self, condition: &IfCondition) -> bool {
        match condition.cond_type.as_str() {
            "binary" => self.evaluate_binary_condition(condition),
            "unary" => self.evaluate_unary_condition(condition),
            "identifier" => self.evaluate_identifier_condition(condition),
            "literal" => self.evaluate_literal_condition(condition),
            _ => {
                warn!("Unknown condition type: {}", condition.cond_type);
                false
            }
        }
    }

    fn evaluate_binary_condition(&self, condition: &IfCondition) -> bool {
        let left = condition.left.as_ref().unwrap();
        let right = condition.right.as_ref().unwrap();
        let operator = condition.operator.as_ref().unwrap();

        let left_value = self.evaluate_condition(left);
        let right_value = self.evaluate_condition(right);

        match operator.as_str() {
            "&&" => left_value && right_value,
            "||" => left_value || right_value,
            ">" => self.compare_values(left, right, |a, b| a > b),
            "<" => self.compare_values(left, right, |a, b| a < b),
            ">=" => self.compare_values(left, right, |a, b| a >= b),
            "<=" => self.compare_values(left, right, |a, b| a <= b),
            "==" => self.compare_values(left, right, |a, b| (a - b).abs() < f64::EPSILON),
            "!=" => self.compare_values(left, right, |a, b| (a - b).abs() >= f64::EPSILON),
            _ => {
                warn!("Unknown binary operator: {}", operator);
                false
            }
        }
    }

    fn evaluate_unary_condition(&self, condition: &IfCondition) -> bool {
        let operand = condition.operand.as_ref().unwrap();
        let operator = condition.operator.as_ref().unwrap();

        match operator.as_str() {
            "!" => !self.evaluate_condition(operand),
            _ => {
                warn!("Unknown unary operator: {}", operator);
                false
            }
        }
    }

    fn evaluate_identifier_condition(&self, condition: &IfCondition) -> bool {
        let identifier = condition.value.as_ref().unwrap();
        match self.get(identifier) {
            Some(MortarVariableValue::Boolean(b)) => *b,
            Some(_) => {
                warn!(
                    "Variable '{}' is not a boolean, cannot evaluate as condition",
                    identifier
                );
                false
            }
            None => {
                warn!("Variable '{}' not found", identifier);
                false
            }
        }
    }

    fn evaluate_literal_condition(&self, condition: &IfCondition) -> bool {
        let value = condition.value.as_ref().unwrap();
        match value.as_str() {
            "true" => true,
            "false" => false,
            _ => {
                warn!("Unknown literal value: {}", value);
                false
            }
        }
    }

    fn compare_values<F>(&self, left: &IfCondition, right: &IfCondition, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_num = self.get_numeric_value(left);
        let right_num = self.get_numeric_value(right);

        match (left_num, right_num) {
            (Some(l), Some(r)) => cmp(l, r),
            _ => {
                warn!("Cannot compare non-numeric values");
                false
            }
        }
    }

    fn get_numeric_value(&self, condition: &IfCondition) -> Option<f64> {
        match condition.cond_type.as_str() {
            "identifier" => {
                let identifier = condition.value.as_ref()?;
                match self.get(identifier) {
                    Some(MortarVariableValue::Number(n)) => Some(*n),
                    _ => None,
                }
            }
            "literal" => {
                let value = condition.value.as_ref()?;
                value.parse::<f64>().ok()
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_state_basic() {
        let mut state = MortarVariableState::new();
        state.set("score", MortarVariableValue::Number(100.0));
        state.set("name", MortarVariableValue::String("Player".to_string()));
        state.set("is_active", MortarVariableValue::Boolean(true));

        assert_eq!(
            state.get("score"),
            Some(&MortarVariableValue::Number(100.0))
        );
        assert_eq!(
            state.get("name"),
            Some(&MortarVariableValue::String("Player".to_string()))
        );
        assert_eq!(
            state.get("is_active"),
            Some(&MortarVariableValue::Boolean(true))
        );
    }

    #[test]
    fn test_evaluate_simple_condition() {
        let mut state = MortarVariableState::new();
        state.set("is_winner", MortarVariableValue::Boolean(true));

        // Test identifier condition
        let condition = IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("is_winner".to_string()),
        };

        assert!(state.evaluate_condition(&condition));
    }

    #[test]
    fn test_evaluate_comparison() {
        let mut state = MortarVariableState::new();
        state.set("score", MortarVariableValue::Number(150.0));

        // Create condition: score > 100
        let condition = IfCondition {
            cond_type: "binary".to_string(),
            operator: Some(">".to_string()),
            left: Some(Box::new(IfCondition {
                cond_type: "identifier".to_string(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("score".to_string()),
            })),
            right: Some(Box::new(IfCondition {
                cond_type: "literal".to_string(),
                operator: None,
                left: None,
                right: None,
                operand: None,
                value: Some("100".to_string()),
            })),
            operand: None,
            value: None,
        };

        assert!(state.evaluate_condition(&condition));
    }
}
