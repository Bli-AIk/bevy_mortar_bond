//! Variable state management for Mortar runtime.
//!
//! Mortar 运行时的变量状态管理。

use bevy::prelude::*;
use mortar_compiler::{Enum, IfCondition, Variable};
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

/// Branch definition for branch interpolation.
///
/// 用于分支插值的分支定义。
#[derive(Debug, Clone)]
struct BranchDef {
    enum_type: Option<String>,
    cases: Vec<BranchCase>,
}

#[derive(Debug, Clone)]
struct BranchCase {
    condition: String,
    text: String,
}

/// Component that manages variable state for a Mortar dialogue runtime.
///
/// 管理 Mortar 对话运行时变量状态的组件。
#[derive(Component, Debug, Clone)]
pub struct MortarVariableState {
    variables: HashMap<String, MortarVariableValue>,
    branches: HashMap<String, BranchDef>,
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
            branches: HashMap::new(),
        }
    }

    /// Initialize from a list of variable declarations.
    ///
    /// 从变量声明列表初始化。
    pub fn from_variables(variables: &[Variable], enums: &[Enum]) -> Self {
        let mut state = Self::new();

        for var in variables {
            // Handle Branch type specially

            if var.var_type == "Branch" {
                if let Some(value) = &var.value
                    && let Some(obj) = value.as_object()
                {
                    let enum_type = obj
                        .get("enum_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let mut cases = Vec::new();

                    if let Some(cases_array) = obj.get("cases").and_then(|v| v.as_array()) {
                        for case in cases_array {
                            if let Some(case_obj) = case.as_object() {
                                let condition = case_obj
                                    .get("condition")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let text = case_obj
                                    .get("text")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                cases.push(BranchCase { condition, text });
                            }
                        }
                    }

                    state
                        .branches
                        .insert(var.name.clone(), BranchDef { enum_type, cases });
                }

                continue;
            }

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

                    // Non-primitive type, check if it's a known enum
                    enum_type_name => {
                        if let Some(enum_def) = enums.iter().find(|e| e.name == enum_type_name) {
                            if let Some(first_member) = enum_def.variants.first() {
                                // Default to the first member of the enum

                                let enum_value = format!("{}.{}", enum_def.name, first_member);

                                MortarVariableValue::String(enum_value)
                            } else {
                                // Enum has no members, skip

                                continue;
                            }
                        } else {
                            // Unknown type, skip

                            continue;
                        }
                    }
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

    /// Execute an assignment statement.
    ///
    /// 执行赋值语句。
    pub fn execute_assignment(&mut self, var_name: &str, value_str: &str) {
        // Parse the value string
        if value_str.contains('.') {
            // Enum member: "EnumName.member"
            self.set(var_name, MortarVariableValue::String(value_str.to_string()));
        } else if value_str == "true" {
            self.set(var_name, MortarVariableValue::Boolean(true));
        } else if value_str == "false" {
            self.set(var_name, MortarVariableValue::Boolean(false));
        } else if let Ok(num) = value_str.parse::<f64>() {
            self.set(var_name, MortarVariableValue::Number(num));
        } else {
            // String or identifier
            self.set(var_name, MortarVariableValue::String(value_str.to_string()));
        }
    }

    /// Set a branch variable's text directly (for testing).
    ///
    /// 直接设置分支变量的文本（用于测试）。
    pub fn set_branch_text(&mut self, name: String, text: String) {
        let branch_def = BranchDef {
            enum_type: None,
            cases: vec![BranchCase {
                condition: "default".to_string(),
                text: text.clone(),
            }],
        };
        self.branches.insert(name.clone(), branch_def);
        // Also set a variable to make the condition true
        self.set("default", MortarVariableValue::Boolean(true));
    }

    /// Get a branch variable's events by evaluating its conditions.
    ///
    /// 通过评估条件获取分支变量的事件。
    pub fn get_branch_events(
        &self,
        name: &str,
        variables: &[Variable],
    ) -> Option<Vec<mortar_compiler::Event>> {
        // Find the branch variable definition
        let branch_var = variables
            .iter()
            .find(|v| v.name == name && v.var_type == "Branch")?;

        // Parse the branch definition
        let value = branch_var.value.as_ref()?;
        let cases = value.get("cases")?.as_array()?;

        // Get enum type if exists
        let enum_type = value.get("enum_type").and_then(|v| v.as_str());

        // Determine which case matches
        let matching_case = if let Some(enum_var_name) = enum_type {
            // Enum-based branch
            if let Some(enum_value) = self.get(enum_var_name) {
                let enum_member = enum_value.to_display_string();
                let member_name = if let Some(dot_pos) = enum_member.rfind('.') {
                    &enum_member[dot_pos + 1..]
                } else {
                    &enum_member
                };

                cases.iter().find(|case| {
                    case.get("condition")
                        .and_then(|c| c.as_str())
                        .map(|c| c == member_name)
                        .unwrap_or(false)
                })
            } else {
                None
            }
        } else {
            // Boolean-based branch
            cases.iter().find(|case| {
                case.get("condition")
                    .and_then(|c| c.as_str())
                    .and_then(|cond_name| self.get(cond_name))
                    .map(|v| matches!(v, MortarVariableValue::Boolean(true)))
                    .unwrap_or(false)
            })
        }?;

        // Extract events from the matching case
        let events_array = matching_case.get("events")?.as_array()?;
        let mut result = Vec::new();

        for event_json in events_array {
            if let Ok(event) = serde_json::from_value::<mortar_compiler::Event>(event_json.clone())
            {
                result.push(event);
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Get a branch variable's text by evaluating its conditions.
    ///
    /// 通过评估条件获取分支变量的文本。
    pub fn get_branch_text(&self, name: &str) -> Option<String> {
        let branch = self.branches.get(name)?;

        // If enum-based branch, check the enum variable value
        if let Some(enum_var_name) = &branch.enum_type {
            // Get the enum variable value (it's stored as "EnumName.member")
            if let Some(enum_value) = self.get(enum_var_name) {
                let enum_member = enum_value.to_display_string();
                // Extract the member name after the dot
                let member_name = if let Some(dot_pos) = enum_member.rfind('.') {
                    &enum_member[dot_pos + 1..]
                } else {
                    &enum_member
                };

                // Find the case that matches the enum member
                for case in &branch.cases {
                    if case.condition == member_name {
                        return Some(case.text.clone());
                    }
                }
            }
        } else {
            // Boolean-based branch: check each condition
            for case in &branch.cases {
                if let Some(condition_value) = self.get(&case.condition)
                    && let MortarVariableValue::Boolean(true) = condition_value
                {
                    return Some(case.text.clone());
                }
            }
        }

        None
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
            "func_call" => self.evaluate_func_call_condition(condition),
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

        match operator.as_str() {
            "&&" | "||" => {
                // For logical operators, recursively evaluate both sides as boolean
                let left_value = self.evaluate_condition(left);
                let right_value = self.evaluate_condition(right);
                match operator.as_str() {
                    "&&" => left_value && right_value,
                    "||" => left_value || right_value,
                    _ => unreachable!(),
                }
            }
            ">" => self.compare_values(left, right, |a, b| a > b),
            "<" => self.compare_values(left, right, |a, b| a < b),
            ">=" => self.compare_values(left, right, |a, b| a >= b),
            "<=" => self.compare_values(left, right, |a, b| a <= b),
            "==" => self.compare_values_eq(left, right, true),
            "!=" => self.compare_values_eq(left, right, false),
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

    fn evaluate_func_call_condition(&self, _condition: &IfCondition) -> bool {
        // Function calls in conditions require access to MortarFunctionRegistry
        // which is not available in MortarVariableState.
        // This should be handled at a higher level where both variable_state and functions are available.
        // For now, we return false and log an info message.
        info!("Function call in condition requires runtime function evaluation");
        false
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

    fn compare_values_eq(
        &self,
        left: &IfCondition,
        right: &IfCondition,
        expect_equal: bool,
    ) -> bool {
        // Try numeric comparison first
        let left_num = self.get_numeric_value(left);
        let right_num = self.get_numeric_value(right);

        if let (Some(l), Some(r)) = (left_num, right_num) {
            let is_equal = (l - r).abs() < f64::EPSILON;
            return if expect_equal { is_equal } else { !is_equal };
        }

        // Try string comparison (for enum members)
        let left_str = self.get_string_value(left);
        let right_str = self.get_string_value(right);

        if let (Some(l), Some(r)) = (left_str, right_str) {
            let is_equal = l == r;
            return if expect_equal { is_equal } else { !is_equal };
        }

        false
    }

    fn get_string_value(&self, condition: &IfCondition) -> Option<String> {
        match condition.cond_type.as_str() {
            "identifier" => {
                let identifier = condition.value.as_ref()?;
                self.get(identifier).map(|val| val.to_display_string())
            }
            "enum_member" => condition.value.clone(),
            "literal" => condition.value.clone(),
            _ => None,
        }
    }

    fn get_numeric_value(&self, condition: &IfCondition) -> Option<f64> {
        match condition.cond_type.as_str() {
            "identifier" => {
                let identifier = condition.value.as_ref()?;
                // First try to parse as a number literal (workaround for serializer issue)
                if let Ok(num) = identifier.parse::<f64>() {
                    return Some(num);
                }
                // Otherwise treat as a variable name
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
