//! Unit tests for bevy_mortar_bond
//!
//! 测试 bevy_mortar_bond 的单元测试

#[cfg(test)]
mod core_tests {
    use crate::*;

    // MortarEventTracker tests.
    //
    // MortarEventTracker 测试。

    #[test]
    fn test_event_tracker_creation() {
        let events = vec![mortar_compiler::Event {
            index: 5.0,
            index_variable: None,
            actions: vec![mortar_compiler::Action {
                action_type: "test_action".to_string(),
                args: vec![],
            }],
        }];

        let tracker = MortarEventTracker::new(events.clone());
        assert_eq!(tracker.event_count(), 1);
        assert_eq!(tracker.fired_count(), 0);
    }

    #[test]
    fn test_event_tracker_trigger_at_index() {
        let events = vec![
            mortar_compiler::Event {
                index: 5.0,
                index_variable: None,
                actions: vec![mortar_compiler::Action {
                    action_type: "play_sound".to_string(),
                    args: vec!["test.wav".to_string()],
                }],
            },
            mortar_compiler::Event {
                index: 10.0,
                index_variable: None,
                actions: vec![mortar_compiler::Action {
                    action_type: "set_color".to_string(),
                    args: vec!["#FF0000".to_string()],
                }],
            },
        ];

        let mut tracker = MortarEventTracker::new(events);
        let runtime = MortarRuntime::default();

        // Before reaching first event.
        //
        // 在到达第一个事件之前。
        let actions = tracker.trigger_at_index(4.0, &runtime);
        assert_eq!(actions.len(), 0);
        assert_eq!(tracker.fired_count(), 0);

        // Reach first event.
        //
        // 触发第一个事件。
        let actions = tracker.trigger_at_index(5.0, &runtime);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_name, "play_sound");
        assert_eq!(tracker.fired_count(), 1);

        // Don't re-trigger first event.
        //
        // 不应重复触发第一个事件。
        let actions = tracker.trigger_at_index(6.0, &runtime);
        assert_eq!(actions.len(), 0);
        assert_eq!(tracker.fired_count(), 1);

        // Trigger second event.
        //
        // 触发第二个事件。
        let actions = tracker.trigger_at_index(10.0, &runtime);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_name, "set_color");
        assert_eq!(tracker.fired_count(), 2);
    }

    #[test]
    fn test_event_tracker_reset() {
        let events = vec![mortar_compiler::Event {
            index: 5.0,
            index_variable: None,
            actions: vec![mortar_compiler::Action {
                action_type: "test".to_string(),
                args: vec![],
            }],
        }];

        let mut tracker = MortarEventTracker::new(events);
        let runtime = MortarRuntime::default();

        // Trigger event.
        //
        // 触发事件。
        let _ = tracker.trigger_at_index(5.0, &runtime);
        assert_eq!(tracker.fired_count(), 1);

        // Reset.
        //
        // 重置。
        tracker.reset();
        assert_eq!(tracker.fired_count(), 0);

        // Can trigger again after reset.
        //
        // 重置后可再次触发。
        let actions = tracker.trigger_at_index(5.0, &runtime);
        assert_eq!(actions.len(), 1);
        assert_eq!(tracker.fired_count(), 1);
    }

    // DialogueState tests.
    //
    // DialogueState 测试。

    fn create_test_node() -> Node {
        use serde_json::json;

        Node {
            name: "TestNode".to_string(),
            content: vec![
                json!({
                    "type": "text",
                    "value": "First text",
                }),
                json!({
                    "type": "text",
                    "value": "Second text",
                }),
            ],
            branches: None,
            variables: vec![],
            next: Some("NextNode".to_string()),
        }
    }

    #[test]
    fn test_dialogue_state_creation() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        assert_eq!(state.mortar_path, "test.mortar");
        assert_eq!(state.current_node, "TestNode");
        assert_eq!(state.text_index, 0);
        assert_eq!(state.selected_choice, None);
        assert_eq!(state.executed_content_indices.len(), 0);
    }

    #[test]
    fn test_dialogue_state_text_navigation() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initial state.
        //
        // 初始状态。
        assert_eq!(state.current_text(), Some("First text"));
        assert!(state.has_next_text());

        // Advance to next text.
        //
        // 前进到下一条文本。
        assert!(state.next_text());
        assert_eq!(state.current_text(), Some("Second text"));
        assert!(!state.has_next_text());

        // Can't advance beyond last text.
        //
        // 不能越过最后一条文本。
        assert!(!state.next_text());
        assert_eq!(state.text_index, 1);
    }

    #[test]
    fn test_dialogue_state_reset() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Advance to second text.
        //
        // 前进到第二条文本。
        state.next_text();
        assert_eq!(state.text_index, 1);

        // Reset.
        //
        // 重置。
        state.reset();
        assert_eq!(state.text_index, 0);
        assert_eq!(state.current_text(), Some("First text"));
    }

    #[test]
    fn test_dialogue_state_choice_stack() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initially empty.
        //
        // 初始为空。
        assert_eq!(state.choice_stack.len(), 0);

        // Push choices.
        //
        // 推入选项。
        state.push_choice(0);
        assert_eq!(state.choice_stack.len(), 1);
        assert_eq!(state.selected_choice, None);

        state.push_choice(1);
        assert_eq!(state.choice_stack.len(), 2);

        // Pop choices.
        //
        // 弹出选项。
        assert_eq!(state.pop_choice(), Some(1));
        assert_eq!(state.choice_stack.len(), 1);

        // Clear stack.
        //
        // 清空堆栈。
        state.clear_choice_stack();
        assert_eq!(state.choice_stack.len(), 0);
    }

    #[test]
    fn test_dialogue_state_content_tracking() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Mark content items as executed.
        //
        // 标记内容项为已执行。
        state.mark_content_executed(0);
        state.mark_content_executed(1);
        assert!(state.executed_content_indices.contains(&0));
        assert!(state.executed_content_indices.contains(&1));

        // Don't duplicate.
        //
        // 不要重复记录。
        state.mark_content_executed(0);
        assert_eq!(
            state
                .executed_content_indices
                .iter()
                .filter(|&&x| x == 0)
                .count(),
            1
        );
    }

    #[test]
    fn test_dialogue_state_has_next_text_before_choice() {
        use serde_json::json;

        // Create node with text, choice, text pattern.
        //
        // 创建包含文本、选项与文本组合的节点。
        let node = Node {
            name: "TestNode".to_string(),
            content: vec![
                json!({
                    "type": "text",
                    "value": "First text",
                }),
                json!({
                    "type": "choice",
                    "options": [
                        {
                            "text": "Option 1",
                            "next": "Node1"
                        }
                    ]
                }),
                json!({
                    "type": "text",
                    "value": "Third text",
                }),
            ],
            branches: None,
            variables: vec![],
            next: Some("NextNode".to_string()),
        };

        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // At text_index 0, there's a choice before the next text.
        //
        // 在 text_index 0 时，下一段文本前存在一个选项。
        assert!(!state.has_next_text_before_choice());
    }

    // MortarValue tests.
    //
    // MortarValue 测试。

    #[test]
    fn test_mortar_value_parse_string() {
        let val = MortarValue::parse("\"hello\"");
        match val {
            MortarValue::String(s) => assert_eq!(s.0, "hello"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_mortar_value_parse_number() {
        let val = MortarValue::parse("42.5");
        match val {
            MortarValue::Number(n) => assert_eq!(n.0, 42.5),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_mortar_value_parse_boolean() {
        let val_true = MortarValue::parse("true");
        match val_true {
            MortarValue::Boolean(b) => assert!(b.0),
            _ => panic!("Expected Boolean"),
        }

        let val_false = MortarValue::parse("false");
        match val_false {
            MortarValue::Boolean(b) => assert!(!b.0),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_mortar_value_to_display_string() {
        assert_eq!(
            MortarValue::String(MortarString("test".to_string())).to_display_string(),
            "test"
        );
        assert_eq!(
            MortarValue::Number(MortarNumber(3.42)).to_display_string(),
            "3.42"
        );
        assert_eq!(
            MortarValue::Boolean(MortarBoolean(true)).to_display_string(),
            "true"
        );
    }

    // MortarVariableState tests.
    //
    // MortarVariableState 测试。

    #[test]
    fn test_variable_state_from_variables() {
        let variables = vec![
            mortar_compiler::Variable {
                name: "health".to_string(),
                var_type: "Number".to_string(),
                value: Some(serde_json::json!(100.0)),
            },
            mortar_compiler::Variable {
                name: "player_name".to_string(),
                var_type: "String".to_string(),
                value: Some(serde_json::json!("Alice")),
            },
        ];

        let state = MortarVariableState::from_variables(&variables, &[], &[]);

        assert_eq!(
            state.get("health"),
            Some(&MortarVariableValue::Number(100.0))
        );
        assert_eq!(
            state.get("player_name"),
            Some(&MortarVariableValue::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_variable_state_execute_assignment() {
        let variables = vec![mortar_compiler::Variable {
            name: "count".to_string(),
            var_type: "Number".to_string(),
            value: Some(serde_json::json!(0.0)),
        }];

        let mut state = MortarVariableState::from_variables(&variables, &[], &[]);

        // Update variable.
        //
        // 更新变量。
        state.execute_assignment("count", "42");
        assert_eq!(state.get("count"), Some(&MortarVariableValue::Number(42.0)));

        // Update string (note: execute_assignment stores the raw value including quotes).
        //
        // 更新字符串（execute_assignment 会保留包含引号的原值）。
        state.execute_assignment("count", "\"text\"");
        assert_eq!(
            state.get("count"),
            Some(&MortarVariableValue::String("\"text\"".to_string()))
        );
    }

    #[test]
    fn test_variable_state_evaluate_condition() {
        let variables = vec![
            mortar_compiler::Variable {
                name: "has_key".to_string(),
                var_type: "Boolean".to_string(),
                value: Some(serde_json::json!(true)),
            },
            mortar_compiler::Variable {
                name: "is_locked".to_string(),
                var_type: "Boolean".to_string(),
                value: Some(serde_json::json!(false)),
            },
        ];

        let state = MortarVariableState::from_variables(&variables, &[], &[]);

        // Test identifier condition (true boolean variable).
        //
        // 测试标识符条件（布尔变量为真）。
        let condition = mortar_compiler::IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("has_key".to_string()),
        };
        assert!(state.evaluate_condition(&condition));

        // Test identifier condition (false boolean variable).
        //
        // 测试标识符条件（布尔变量为假）。
        let condition = mortar_compiler::IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("is_locked".to_string()),
        };
        assert!(!state.evaluate_condition(&condition));
    }

    #[test]
    fn test_variable_value_to_display_string() {
        assert_eq!(
            MortarVariableValue::Number(3.42).to_display_string(),
            "3.42"
        );
        assert_eq!(
            MortarVariableValue::String("test".to_string()).to_display_string(),
            "test"
        );
        assert_eq!(
            MortarVariableValue::Boolean(true).to_display_string(),
            "true"
        );
    }

    // MortarRegistry tests.
    //
    // MortarRegistry 测试。

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = MortarRegistry::default();
        let handle: Handle<MortarAsset> = Handle::default();

        // Register asset.
        //
        // 注册资源。
        registry.register("test.mortar", handle.clone());

        // Retrieve asset.
        //
        // 获取资源。
        assert!(registry.get("test.mortar").is_some());
        assert!(registry.get("nonexistent.mortar").is_none());
    }

    // Interpolation tests.
    //
    // 插值测试。

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

    // Integration tests.
    //
    // 集成测试。

    // Disabled: Needs update for new content structure.
    //
    // 已禁用：需要根据新的内容结构更新。

    // Disabled: Needs update for new content structure.
    //
    // 已禁用：需要根据新的内容结构更新。

    #[test]
    fn test_event_with_index_variable() {
        let event = mortar_compiler::Event {
            index: 0.0,
            index_variable: Some("custom_time".to_string()),
            actions: vec![mortar_compiler::Action {
                action_type: "play_sound".to_string(),
                args: vec!["test.wav".to_string()],
            }],
        };

        assert_eq!(event.index_variable, Some("custom_time".to_string()));
    }

    // MortarFunctionRegistry tests.
    //
    // MortarFunctionRegistry 测试。

    #[test]
    fn test_function_registry_register_and_call() {
        let mut registry = MortarFunctionRegistry::new();

        // Register a test function.
        //
        // 注册一个测试函数。
        registry.register("test_func", |args| {
            if let Some(MortarValue::String(s)) = args.first() {
                MortarValue::String(MortarString(format!("Result: {}", s.0)))
            } else {
                MortarValue::Void
            }
        });

        // Call the function.
        //
        // 调用该函数。
        let result = registry.call(
            "test_func",
            &[MortarValue::String(MortarString("hello".to_string()))],
        );
        assert!(result.is_some());
        match result.unwrap() {
            MortarValue::String(s) => assert_eq!(s.0, "Result: hello"),
            _ => panic!("Expected String result"),
        }

        // Call non-existent function.
        //
        // 调用不存在的函数。
        let result = registry.call("non_existent", &[]);
        assert!(result.is_none());
    }

    // Additional DialogueState tests.
    //
    // 额外的 DialogueState 测试。

    #[test]
    fn test_dialogue_state_get_next_node() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        assert_eq!(state.get_next_node(), Some("NextNode"));
    }

    // Disabled: Needs update for new content structure.
    //
    // 已禁用：需要根据新的内容结构更新。

    // Disabled: Needs complete rewrite for new content structure.
    //
    // 已禁用：需要依据新的内容结构彻底重写。
    // Use get_runs_at_content_position instead of get_runs_at_position.
    //
    // 请改用 get_runs_at_content_position 取代 get_runs_at_position。

    // Variable state advanced tests.
    //
    // 变量状态高级测试。

    #[test]
    fn test_variable_state_branch_text() {
        let mut state = MortarVariableState::default();

        state.set_branch_text("branch1".to_string(), "Option A".to_string());
        state.set_branch_text("branch2".to_string(), "Option B".to_string());

        assert_eq!(
            state.get_branch_text("branch1"),
            Some("Option A".to_string())
        );
        assert_eq!(
            state.get_branch_text("branch2"),
            Some("Option B".to_string())
        );
        assert_eq!(state.get_branch_text("nonexistent"), None);
    }

    #[test]
    fn test_variable_state_parse_value() {
        let variables = vec![mortar_compiler::Variable {
            name: "test".to_string(),
            var_type: "Number".to_string(),
            value: Some(serde_json::json!(42.0)),
        }];

        let mut state = MortarVariableState::from_variables(&variables, &[], &[]);

        // Parse number.
        //
        // 解析数字。
        state.execute_assignment("test", "100");
        assert_eq!(state.get("test"), Some(&MortarVariableValue::Number(100.0)));

        // Parse boolean.
        //
        // 解析布尔值。
        state.execute_assignment("test", "true");
        assert_eq!(state.get("test"), Some(&MortarVariableValue::Boolean(true)));

        state.execute_assignment("test", "false");
        assert_eq!(
            state.get("test"),
            Some(&MortarVariableValue::Boolean(false))
        );
    }

    // Condition evaluation tests.
    //
    // 条件求值测试。

    #[test]
    fn test_evaluate_condition_with_function() {
        let functions = MortarFunctionRegistry::new();
        let function_decls = vec![];

        let condition = mortar_compiler::Condition {
            condition_type: "has_key".to_string(),
            args: vec![],
        };

        // Function not found should return false.
        //
        // 找不到函数时应返回 false。
        let result = evaluate_condition(&condition, &functions, &function_decls);
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_with_registered_function() {
        let mut functions = MortarFunctionRegistry::new();

        // Register a condition function.
        //
        // 注册条件函数。
        functions.register("has_key", |_args| MortarValue::Boolean(MortarBoolean(true)));

        let function_decls = vec![];
        let condition = mortar_compiler::Condition {
            condition_type: "has_key".to_string(),
            args: vec![],
        };

        let result = evaluate_condition(&condition, &functions, &function_decls);
        assert!(result);
    }

    // MortarValue advanced tests.
    //
    // MortarValue 高级测试。

    #[test]
    fn test_mortar_value_parse_edge_cases() {
        // Parse empty string.
        //
        // 解析空字符串。
        let val = MortarValue::parse("");
        match val {
            MortarValue::String(s) => assert_eq!(s.0, ""),
            _ => panic!("Expected String"),
        }

        // Parse quoted empty string.
        //
        // 解析带引号的空字符串。
        let val = MortarValue::parse("\"\"");
        match val {
            MortarValue::String(s) => assert_eq!(s.0, ""),
            _ => panic!("Expected String"),
        }

        // Parse negative number.
        //
        // 解析负数。
        let val = MortarValue::parse("-42.5");
        match val {
            MortarValue::Number(n) => assert_eq!(n.0, -42.5),
            _ => panic!("Expected Number"),
        }

        // Parse zero.
        //
        // 解析零值。
        let val = MortarValue::parse("0");
        match val {
            MortarValue::Number(n) => assert_eq!(n.0, 0.0),
            _ => panic!("Expected Number"),
        }
    }

    // Text data tests.
    //
    // 文本数据测试。

    #[test]
    fn test_dialogue_state_current_text_data() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        let text_data = state.current_text_data();
        assert!(text_data.is_some());
        assert_eq!(text_data.unwrap().value, "First text");
    }

    #[test]
    fn test_dialogue_state_choices_broken() {
        use serde_json::json;

        let node = Node {
            name: "TestNode".to_string(),
            content: vec![
                json!({
                    "type": "text",
                    "value": "Text before choice",
                }),
                json!({
                    "type": "choice",
                    "options": [
                        {
                            "text": "Test",
                            "next": "Next"
                        }
                    ]
                }),
            ],
            branches: None,
            variables: vec![],
            next: None,
        };

        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initially choices not broken.
        //
        // 初始状态下选项未被破坏。
        assert!(!state.choices_broken);
        assert!(state.get_choices().is_some());

        // Break choices.
        //
        // 破坏选项。
        state.choices_broken = true;
        assert!(state.get_choices().is_none());
    }

    // --- Line group tests ---

    fn create_line_group_node() -> mortar_compiler::Node {
        use serde_json::json;
        mortar_compiler::Node {
            name: "LineGroupNode".to_string(),
            content: vec![
                json!({ "type": "line", "value": "Line A" }),
                json!({ "type": "line", "value": "Line B" }),
                json!({ "type": "line", "value": "Line C" }),
            ],
            branches: None,
            variables: vec![],
            next: None,
        }
    }

    #[test]
    fn test_line_group_is_line_flag() {
        let node = create_line_group_node();
        let state =
            DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

        let td = state.current_text_data().unwrap();
        assert!(td.is_line, "First entry should be is_line=true");
    }

    #[test]
    fn test_line_group_end_covers_consecutive_lines() {
        let node = create_line_group_node();
        let state =
            DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

        let group = state.current_line_group().unwrap();
        assert_eq!(
            group.len(),
            3,
            "All 3 consecutive lines should form one group"
        );
        assert_eq!(group[0].value, "Line A");
        assert_eq!(group[1].value, "Line B");
        assert_eq!(group[2].value, "Line C");
    }

    #[test]
    fn test_line_group_next_text_skips_entire_group() {
        let node = create_line_group_node();
        let mut state =
            DialogueState::new("test.mortar".to_string(), "LineGroupNode".to_string(), node);

        // Should not have next text — the entire node is one line group
        assert!(
            !state.has_next_text(),
            "3-line group should be the only step"
        );

        // next_text should return false (no more text after the group)
        assert!(
            !state.next_text(),
            "next_text should return false when no text after group"
        );
    }

    #[test]
    fn test_mixed_text_and_line_group() {
        use serde_json::json;
        let node = mortar_compiler::Node {
            name: "MixedNode".to_string(),
            content: vec![
                json!({ "type": "text", "value": "Step 1" }),
                json!({ "type": "line", "value": "Line A" }),
                json!({ "type": "line", "value": "Line B" }),
                json!({ "type": "text", "value": "Step 2" }),
            ],
            branches: None,
            variables: vec![],
            next: None,
        };
        let mut state =
            DialogueState::new("test.mortar".to_string(), "MixedNode".to_string(), node);

        // Step 1: regular text
        assert_eq!(state.current_text(), Some("Step 1"));
        assert!(!state.current_text_data().unwrap().is_line);
        assert!(state.has_next_text());

        // Advance: text_index 0 → 1
        assert!(state.next_text());

        // Step 2: line group (Line A + Line B)
        let td = state.current_text_data().unwrap();
        assert!(td.is_line);
        let group = state.current_line_group().unwrap();
        assert_eq!(group.len(), 2);
        assert_eq!(group[0].value, "Line A");
        assert_eq!(group[1].value, "Line B");
        assert!(state.has_next_text());

        // Advance: text_index 1 → 3 (skip entire line group)
        assert!(state.next_text());

        // Step 3: regular text
        assert_eq!(state.current_text(), Some("Step 2"));
        assert!(!state.current_text_data().unwrap().is_line);
        assert!(!state.has_next_text());
    }

    #[test]
    fn test_line_group_with_conditions() {
        use serde_json::json;
        let node = mortar_compiler::Node {
            name: "CondNode".to_string(),
            content: vec![
                json!({ "type": "line", "value": "Always shown" }),
                json!({
                    "type": "line",
                    "value": "Conditional line",
                    "condition": {
                        "type": "identifier",
                        "value": "some_flag"
                    }
                }),
            ],
            branches: None,
            variables: vec![],
            next: None,
        };
        let state = DialogueState::new("test.mortar".to_string(), "CondNode".to_string(), node);

        // Both entries should still be in the same line group
        let group = state.current_line_group().unwrap();
        assert_eq!(
            group.len(),
            2,
            "Conditional line is still part of the group"
        );
        assert!(group[0].condition.is_none());
        assert!(group[1].condition.is_some());

        // The entire group is one step
        assert!(
            !state.has_next_text(),
            "Conditional line group should be one step"
        );
    }

    #[test]
    fn test_line_group_last_content_index() {
        use serde_json::json;
        let node = mortar_compiler::Node {
            name: "RunAfterLine".to_string(),
            content: vec![
                json!({ "type": "line", "value": "Line A" }), // content 0
                json!({ "type": "line", "value": "Line B" }), // content 1
                json!({ "type": "line", "value": "Line C" }), // content 2
                json!({ "type": "run_event", "name": "sfx" }), // content 3
                json!({ "type": "text", "value": "After group" }), // content 4
            ],
            branches: None,
            variables: vec![],
            next: None,
        };
        let state = DialogueState::new("test.mortar".to_string(), "RunAfterLine".to_string(), node);

        // line_group_last_content_index should point to content index 2 (the last line)
        assert_eq!(
            state.line_group_last_content_index(),
            Some(2),
            "Last line in group should have content index 2"
        );

        // current_text_content_index should point to content index 0 (the first line)
        assert_eq!(
            state.current_text_content_index(),
            Some(0),
            "First line in group should have content index 0"
        );
    }

    #[test]
    fn test_line_group_single_text_last_content_index() {
        use serde_json::json;
        let node = mortar_compiler::Node {
            name: "SingleText".to_string(),
            content: vec![json!({ "type": "text", "value": "Hello" })],
            branches: None,
            variables: vec![],
            next: None,
        };
        let state = DialogueState::new("test.mortar".to_string(), "SingleText".to_string(), node);

        // For a single text, both should return the same index
        assert_eq!(state.current_text_content_index(), Some(0));
        assert_eq!(state.line_group_last_content_index(), Some(0));
    }

    // Integration test: full workflow.
    //
    // 集成测试：完整流程。

    // Disabled: Needs complete rewrite for new content structure.
    //
    // 已禁用：需要依据新的内容结构彻底重写。
}

mod fuzz_tests;
