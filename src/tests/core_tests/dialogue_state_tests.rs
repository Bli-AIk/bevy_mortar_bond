use super::*;

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

    assert_eq!(state.current_text(), Some("First text"));
    assert!(state.has_next_text());

    assert!(state.next_text());
    assert_eq!(state.current_text(), Some("Second text"));
    assert!(!state.has_next_text());

    assert!(!state.next_text());
    assert_eq!(state.text_index, 1);
}

#[test]
fn test_dialogue_state_reset() {
    let node = create_test_node();
    let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

    state.next_text();
    assert_eq!(state.text_index, 1);

    state.reset();
    assert_eq!(state.text_index, 0);
    assert_eq!(state.current_text(), Some("First text"));
}

#[test]
fn test_dialogue_state_choice_stack() {
    let node = create_test_node();
    let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

    assert_eq!(state.choice_stack.len(), 0);

    state.push_choice(0);
    assert_eq!(state.choice_stack.len(), 1);
    assert_eq!(state.selected_choice, None);

    state.push_choice(1);
    assert_eq!(state.choice_stack.len(), 2);

    assert_eq!(state.pop_choice(), Some(1));
    assert_eq!(state.choice_stack.len(), 1);

    state.clear_choice_stack();
    assert_eq!(state.choice_stack.len(), 0);
}

#[test]
fn test_dialogue_state_content_tracking() {
    let node = create_test_node();
    let mut state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

    state.mark_content_executed(0);
    state.mark_content_executed(1);
    assert!(state.executed_content_indices.contains(&0));
    assert!(state.executed_content_indices.contains(&1));

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
    assert!(!state.has_next_text_before_choice());
}

#[test]
fn test_dialogue_state_get_next_node() {
    let node = create_test_node();
    let state = DialogueState::new("test.mortar".to_string(), "TestNode".to_string(), node);

    assert_eq!(state.get_next_node(), Some("NextNode"));
}

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

    assert!(!state.choices_broken);
    assert!(state.get_choices().is_some());

    state.choices_broken = true;
    assert!(state.get_choices().is_none());
}
