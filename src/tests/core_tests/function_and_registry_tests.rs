//! Covers the registry and function-binding primitives at the heart of
//! `bevy_mortar_bond`. These tests make sure mortar assets can be registered,
//! lookup remains stable, and host functions keep the expected calling contract.
//!
//! 覆盖 `bevy_mortar_bond` 最核心的注册表和函数绑定基元。它保证 mortar
//! 资源可以被正确注册、查询行为保持稳定，同时确保宿主函数的调用约定不会在重构中漂移。

use super::*;

#[test]
fn test_registry_register_and_get() {
    let mut registry = MortarRegistry::default();
    let handle: Handle<MortarAsset> = Handle::default();

    registry.register("test.mortar", handle.clone());

    assert!(registry.get("test.mortar").is_some());
    assert!(registry.get("nonexistent.mortar").is_none());
}

#[test]
fn test_function_registry_register_and_call() {
    let mut registry = MortarFunctionRegistry::new();

    registry.register("test_func", |args| {
        if let Some(MortarValue::String(s)) = args.first() {
            MortarValue::String(MortarString(format!("Result: {}", s.0)))
        } else {
            MortarValue::Void
        }
    });

    let result = registry.call(
        "test_func",
        &[MortarValue::String(MortarString("hello".to_string()))],
    );
    assert!(result.is_some());
    match result.unwrap() {
        MortarValue::String(s) => assert_eq!(s.0, "Result: hello"),
        _ => panic!("Expected String result"),
    }

    let result = registry.call("non_existent", &[]);
    assert!(result.is_none());
}

#[test]
fn test_evaluate_condition_with_function() {
    let functions = MortarFunctionRegistry::new();
    let function_decls = vec![];

    let condition = mortar_compiler::Condition {
        condition_type: "has_key".to_string(),
        args: vec![],
    };

    let result = evaluate_condition(&condition, &functions, &function_decls);
    assert!(!result);
}

#[test]
fn test_evaluate_condition_with_registered_function() {
    let mut functions = MortarFunctionRegistry::new();

    functions.register("has_key", |_args| MortarValue::Boolean(MortarBoolean(true)));

    let function_decls = vec![];
    let condition = mortar_compiler::Condition {
        condition_type: "has_key".to_string(),
        args: vec![],
    };

    let result = evaluate_condition(&condition, &functions, &function_decls);
    assert!(result);
}
