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
