use crate::*;

mod dialogue_state_tests;
mod event_tracker_tests;
mod function_and_registry_tests;
mod text_processing_tests;
mod value_and_variable_tests;

pub(super) fn create_test_node() -> Node {
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
