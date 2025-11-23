//! Unit tests for bevy_mortar_bond
//!
//! 测试 bevy_mortar_bond 的单元测试

#[cfg(test)]
mod core_tests {
    use crate::*;

    // =============================================================================
    // MortarEventTracker Tests
    // =============================================================================

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

        // Before reaching first event
        let actions = tracker.trigger_at_index(4, &runtime);
        assert_eq!(actions.len(), 0);
        assert_eq!(tracker.fired_count(), 0);

        // Reach first event
        let actions = tracker.trigger_at_index(5, &runtime);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_name, "play_sound");
        assert_eq!(tracker.fired_count(), 1);

        // Don't re-trigger first event
        let actions = tracker.trigger_at_index(6, &runtime);
        assert_eq!(actions.len(), 0);
        assert_eq!(tracker.fired_count(), 1);

        // Trigger second event
        let actions = tracker.trigger_at_index(10, &runtime);
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

        // Trigger event
        let _ = tracker.trigger_at_index(5, &runtime);
        assert_eq!(tracker.fired_count(), 1);

        // Reset
        tracker.reset();
        assert_eq!(tracker.fired_count(), 0);

        // Can trigger again after reset
        let actions = tracker.trigger_at_index(5, &runtime);
        assert_eq!(actions.len(), 1);
        assert_eq!(tracker.fired_count(), 1);
    }

    // =============================================================================
    // DialogueState Tests
    // =============================================================================

    fn create_test_node() -> Node {
        Node {
            name: "TestNode".to_string(),
            texts: vec![
                mortar_compiler::Text {
                    text: "First text".to_string(),
                    interpolated_parts: None,
                    condition: None,
                    events: None,
                    pre_statements: vec![],
                },
                mortar_compiler::Text {
                    text: "Second text".to_string(),
                    interpolated_parts: None,
                    condition: None,
                    events: None,
                    pre_statements: vec![],
                },
            ],
            branches: None,
            variables: vec![],
            runs: vec![],
            next: Some("NextNode".to_string()),
            choice: None,
            choice_position: None,
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
        assert_eq!(state.executed_runs.len(), 0);
    }

    #[test]
    fn test_dialogue_state_text_navigation() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initial state
        assert_eq!(state.current_text(), Some("First text"));
        assert!(state.has_next_text());

        // Advance to next text
        assert!(state.next_text());
        assert_eq!(state.current_text(), Some("Second text"));
        assert!(!state.has_next_text());

        // Can't advance beyond last text
        assert!(!state.next_text());
        assert_eq!(state.text_index, 1);
    }

    #[test]
    fn test_dialogue_state_reset() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Advance to second text
        state.next_text();
        assert_eq!(state.text_index, 1);

        // Reset
        state.reset();
        assert_eq!(state.text_index, 0);
        assert_eq!(state.current_text(), Some("First text"));
    }

    #[test]
    fn test_dialogue_state_choice_stack() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initially empty
        assert_eq!(state.choice_stack.len(), 0);

        // Push choices
        state.push_choice(0);
        assert_eq!(state.choice_stack.len(), 1);
        assert_eq!(state.selected_choice, None);

        state.push_choice(1);
        assert_eq!(state.choice_stack.len(), 2);

        // Pop choices
        assert_eq!(state.pop_choice(), Some(1));
        assert_eq!(state.choice_stack.len(), 1);

        // Clear stack
        state.clear_choice_stack();
        assert_eq!(state.choice_stack.len(), 0);
    }

    #[test]
    fn test_dialogue_state_run_tracking() {
        let node = create_test_node();
        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Mark runs as executed
        state.mark_run_executed(0);
        state.mark_run_executed(1);
        assert!(state.executed_runs.contains(&0));
        assert!(state.executed_runs.contains(&1));

        // Don't duplicate
        state.mark_run_executed(0);
        assert_eq!(state.executed_runs.iter().filter(|&&x| x == 0).count(), 1);
    }

    #[test]
    fn test_dialogue_state_has_next_text_before_choice() {
        let mut node = create_test_node();
        node.choice_position = Some(1);
        node.texts.push(mortar_compiler::Text {
            text: "Third text".to_string(),
            interpolated_parts: None,
            condition: None,
            events: None,
            pre_statements: vec![],
        });

        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // text_index = 0, choice_position = 1
        // text_index + 1 (1) < choice_position (1) = false
        assert!(!state.has_next_text_before_choice());
    }

    // =============================================================================
    // MortarValue Tests
    // =============================================================================

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

    // =============================================================================
    // MortarVariableState Tests
    // =============================================================================

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

        let state = MortarVariableState::from_variables(&variables);

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

        let mut state = MortarVariableState::from_variables(&variables);

        // Update variable
        state.execute_assignment("count", "42");
        assert_eq!(state.get("count"), Some(&MortarVariableValue::Number(42.0)));

        // Update string (note: execute_assignment stores the raw value including quotes)
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

        let state = MortarVariableState::from_variables(&variables);

        // Test identifier condition (true boolean variable)
        let condition = mortar_compiler::IfCondition {
            cond_type: "identifier".to_string(),
            operator: None,
            left: None,
            right: None,
            operand: None,
            value: Some("has_key".to_string()),
        };
        assert!(state.evaluate_condition(&condition));

        // Test identifier condition (false boolean variable)
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
            MortarVariableValue::Number(3.14).to_display_string(),
            "3.14"
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

    // =============================================================================
    // MortarRegistry Tests
    // =============================================================================

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = MortarRegistry::default();
        let handle: Handle<MortarAsset> = Handle::default();

        // Register asset
        registry.register("test.mortar", handle.clone());

        // Retrieve asset
        assert!(registry.get("test.mortar").is_some());
        assert!(registry.get("nonexistent.mortar").is_none());
    }

    // =============================================================================
    // Interpolation Tests
    // =============================================================================

    #[test]
    fn test_process_interpolated_text_no_interpolation() {
        let text_data = mortar_compiler::Text {
            text: "Plain text".to_string(),
            interpolated_parts: None,
            condition: None,
            events: None,
            pre_statements: vec![],
        };

        let functions = MortarFunctionRegistry::new();
        let function_decls = vec![];
        let var_state = MortarVariableState::default();

        let result = process_interpolated_text(&text_data, &functions, &function_decls, &var_state);
        assert_eq!(result, "Plain text");
    }

    #[test]
    fn test_process_interpolated_text_with_variables() {
        let text_data = mortar_compiler::Text {
            text: "Hello {name}!".to_string(),
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
        let var_state = MortarVariableState::from_variables(&variables);

        let result = process_interpolated_text(&text_data, &functions, &function_decls, &var_state);
        assert_eq!(result, "Hello Alice!");
    }

    // =============================================================================
    // Integration Tests
    // =============================================================================

    #[test]
    fn test_dialogue_flow_with_choices() {
        let mut node = create_test_node();
        node.choice = Some(vec![
            mortar_compiler::Choice {
                text: "Option 1".to_string(),
                condition: None,
                next: Some("Node1".to_string()),
                action: None,
                choice: None,
            },
            mortar_compiler::Choice {
                text: "Option 2".to_string(),
                condition: None,
                next: Some("Node2".to_string()),
                action: None,
                choice: None,
            },
        ]);

        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        assert!(state.has_choices());
        assert_eq!(state.get_choices().unwrap().len(), 2);
    }

    #[test]
    fn test_dialogue_state_with_choice_position() {
        let mut node = create_test_node();
        node.choice_position = Some(1);
        node.choice = Some(vec![mortar_compiler::Choice {
            text: "Test".to_string(),
            condition: None,
            next: Some("Next".to_string()),
            action: None,
            choice: None,
        }]);

        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // At text_index = 0, we have reached choice_position = 1
        assert_eq!(state.text_index, 0);
        assert_eq!(state.node_data().choice_position, Some(1));
    }

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

    // =============================================================================
    // MortarFunctionRegistry Tests
    // =============================================================================

    #[test]
    fn test_function_registry_register_and_call() {
        let mut registry = MortarFunctionRegistry::new();

        // Register a test function
        registry.register("test_func", |args| {
            if let Some(MortarValue::String(s)) = args.first() {
                MortarValue::String(MortarString(format!("Result: {}", s.0)))
            } else {
                MortarValue::Void(MortarVoid)
            }
        });

        // Call the function
        let result = registry.call(
            "test_func",
            &[MortarValue::String(MortarString("hello".to_string()))],
        );
        assert!(result.is_some());
        match result.unwrap() {
            MortarValue::String(s) => assert_eq!(s.0, "Result: hello"),
            _ => panic!("Expected String result"),
        }

        // Call non-existent function
        let result = registry.call("non_existent", &[]);
        assert!(result.is_none());
    }

    // =============================================================================
    // Additional DialogueState Tests
    // =============================================================================

    #[test]
    fn test_dialogue_state_get_next_node() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        assert_eq!(state.get_next_node(), Some("NextNode"));
    }

    #[test]
    fn test_dialogue_state_node_data() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        let node_data = state.node_data();
        assert_eq!(node_data.name, "TestNode");
        assert_eq!(node_data.texts.len(), 2);
    }

    #[test]
    fn test_dialogue_state_get_runs_at_position() {
        let mut node = create_test_node();
        node.runs = vec![
            mortar_compiler::RunStmt {
                event_name: "TestRun1".to_string(),
                args: vec![],
                index_override: None,
                ignore_duration: false,
                position: 0,
            },
            mortar_compiler::RunStmt {
                event_name: "TestRun2".to_string(),
                args: vec![],
                index_override: None,
                ignore_duration: false,
                position: 1,
            },
        ];

        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        let runs = state.get_runs_at_position(0);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].event_name, "TestRun1");

        let runs = state.get_runs_at_position(1);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].event_name, "TestRun2");

        let runs = state.get_runs_at_position(99);
        assert_eq!(runs.len(), 0);
    }

    // =============================================================================
    // Variable State Advanced Tests
    // =============================================================================

    #[test]
    fn test_variable_state_set_branch_text() {
        let mut state = MortarVariableState::default();

        state.set_branch_text("branch1", "Option A");
        state.set_branch_text("branch2", "Option B");

        assert_eq!(state.get_branch_text("branch1"), Some("Option A"));
        assert_eq!(state.get_branch_text("branch2"), Some("Option B"));
        assert_eq!(state.get_branch_text("nonexistent"), None);
    }

    #[test]
    fn test_variable_state_parse_value() {
        let variables = vec![mortar_compiler::Variable {
            name: "test".to_string(),
            var_type: "Number".to_string(),
            value: Some(serde_json::json!(42.0)),
        }];

        let mut state = MortarVariableState::from_variables(&variables);

        // Parse number
        state.execute_assignment("test", "100");
        assert_eq!(state.get("test"), Some(&MortarVariableValue::Number(100.0)));

        // Parse boolean
        state.execute_assignment("test", "true");
        assert_eq!(state.get("test"), Some(&MortarVariableValue::Boolean(true)));

        state.execute_assignment("test", "false");
        assert_eq!(
            state.get("test"),
            Some(&MortarVariableValue::Boolean(false))
        );
    }

    // =============================================================================
    // Condition Evaluation Tests
    // =============================================================================

    #[test]
    fn test_evaluate_condition_with_function() {
        let functions = MortarFunctionRegistry::new();
        let function_decls = vec![];

        let condition = mortar_compiler::Condition {
            condition_type: "has_key".to_string(),
            args: vec![],
        };

        // Function not found should return false
        let result = evaluate_condition(&condition, &functions, &function_decls);
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_with_registered_function() {
        let mut functions = MortarFunctionRegistry::new();

        // Register a condition function
        functions.register("has_key", |_args| MortarValue::Boolean(MortarBoolean(true)));

        let function_decls = vec![];
        let condition = mortar_compiler::Condition {
            condition_type: "has_key".to_string(),
            args: vec![],
        };

        let result = evaluate_condition(&condition, &functions, &function_decls);
        assert!(result);
    }

    // =============================================================================
    // MortarValue Advanced Tests
    // =============================================================================

    #[test]
    fn test_mortar_value_parse_edge_cases() {
        // Parse empty string
        let val = MortarValue::parse("");
        match val {
            MortarValue::String(s) => assert_eq!(s.0, ""),
            _ => panic!("Expected String"),
        }

        // Parse quoted empty string
        let val = MortarValue::parse("\"\"");
        match val {
            MortarValue::String(s) => assert_eq!(s.0, ""),
            _ => panic!("Expected String"),
        }

        // Parse negative number
        let val = MortarValue::parse("-42.5");
        match val {
            MortarValue::Number(n) => assert_eq!(n.0, -42.5),
            _ => panic!("Expected Number"),
        }

        // Parse zero
        let val = MortarValue::parse("0");
        match val {
            MortarValue::Number(n) => assert_eq!(n.0, 0.0),
            _ => panic!("Expected Number"),
        }
    }

    // =============================================================================
    // Text Data Tests
    // =============================================================================

    #[test]
    fn test_dialogue_state_current_text_data() {
        let node = create_test_node();
        let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        let text_data = state.current_text_data();
        assert!(text_data.is_some());
        assert_eq!(text_data.unwrap().text, "First text");
    }

    #[test]
    fn test_dialogue_state_choices_broken() {
        let mut node = create_test_node();
        node.choice = Some(vec![mortar_compiler::Choice {
            text: "Test".to_string(),
            condition: None,
            next: Some("Next".to_string()),
            action: None,
            choice: None,
        }]);

        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Initially choices not broken
        assert!(!state.choices_broken);
        assert!(state.get_choices().is_some());

        // Break choices
        state.choices_broken = true;
        assert!(state.get_choices().is_none());
    }

    // =============================================================================
    // Integration Test: Full Workflow
    // =============================================================================

    #[test]
    fn test_full_dialogue_workflow() {
        // Create a node with multiple texts and choices
        let mut node = Node {
            name: "TestNode".to_string(),
            texts: vec![
                mortar_compiler::Text {
                    text: "First".to_string(),
                    interpolated_parts: None,
                    condition: None,
                    events: Some(vec![mortar_compiler::Event {
                        index: 3.0,
                        index_variable: None,
                        actions: vec![mortar_compiler::Action {
                            action_type: "test_action".to_string(),
                            args: vec![],
                        }],
                    }]),
                    pre_statements: vec![],
                },
                mortar_compiler::Text {
                    text: "Second".to_string(),
                    interpolated_parts: None,
                    condition: None,
                    events: None,
                    pre_statements: vec![],
                },
            ],
            branches: None,
            variables: vec![],
            runs: vec![],
            next: Some("NextNode".to_string()),
            choice: Some(vec![mortar_compiler::Choice {
                text: "Continue".to_string(),
                condition: None,
                next: Some("NextNode".to_string()),
                action: None,
                choice: None,
            }]),
            choice_position: Some(2),
        };

        let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

        // Navigate through texts
        assert_eq!(state.text_index, 0);
        assert_eq!(state.current_text(), Some("First"));
        assert!(state.has_next_text());

        state.next_text();
        assert_eq!(state.text_index, 1);
        assert_eq!(state.current_text(), Some("Second"));
        assert!(!state.has_next_text());

        // Test choices
        assert!(state.has_choices());
        assert_eq!(state.get_choices().unwrap().len(), 1);

        // Select and confirm choice
        state.selected_choice = Some(0);
        assert_eq!(state.selected_choice, Some(0));
    }
}
