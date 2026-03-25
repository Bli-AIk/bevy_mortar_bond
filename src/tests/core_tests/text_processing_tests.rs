//! Tests Mortar's interpolated-text processing path.
//! It keeps the rules for plain text, variable interpolation, and function-backed
//! substitutions pinned down so dialogue rendering changes do not silently alter
//! authored line output.
//!
//! 测试 Mortar 的插值文本处理路径。它把纯文本、变量插值和函数替换等规则
//! 固定下来，避免对话渲染相关改动在不知不觉中改变作者写下的文本输出。

use super::*;

#[test]
fn test_process_interpolated_text_no_interpolation() {
    let text_data = TextData {
        value: "Plain text".to_string(),
        interpolated_parts: None,
        condition: None,
        events: None,
        pre_statements: vec![],
        is_line: false,
    };

    let functions = MortarFunctionRegistry::new();
    let function_decls = vec![];
    let var_state = MortarVariableState::default();

    let result = process_interpolated_text(&text_data, &functions, &function_decls, &var_state);
    assert_eq!(result, "Plain text");
}

#[test]
fn test_process_interpolated_text_with_variables() {
    let text_data = TextData {
        is_line: false,
        value: "Hello {name}!".to_string(),
        interpolated_parts: Some(vec![
            mortar_compiler::StringPart {
                part_type: "text".to_string(),
                content: "Hello ".to_string(),
                function_name: None,
                args: vec![],
                enum_type: None,
                branches: None,
            },
            mortar_compiler::StringPart {
                part_type: "placeholder".to_string(),
                content: "{name}".to_string(),
                function_name: None,
                args: vec![],
                enum_type: None,
                branches: None,
            },
            mortar_compiler::StringPart {
                part_type: "text".to_string(),
                content: "!".to_string(),
                function_name: None,
                args: vec![],
                enum_type: None,
                branches: None,
            },
        ]),
        condition: None,
        events: None,
        pre_statements: vec![],
    };

    let functions = MortarFunctionRegistry::new();
    let function_decls = vec![];

    let variables = vec![mortar_compiler::Variable {
        name: "name".to_string(),
        var_type: "String".to_string(),
        value: Some(serde_json::json!("Alice")),
    }];
    let var_state = MortarVariableState::from_variables(&variables, &[], &[]);

    let result = process_interpolated_text(&text_data, &functions, &function_decls, &var_state);
    assert_eq!(result, "Hello Alice!");
}
