use super::*;

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

    let actions = tracker.trigger_at_index(4.0, &runtime);
    assert_eq!(actions.len(), 0);
    assert_eq!(tracker.fired_count(), 0);

    let actions = tracker.trigger_at_index(5.0, &runtime);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action_name, "play_sound");
    assert_eq!(tracker.fired_count(), 1);

    let actions = tracker.trigger_at_index(6.0, &runtime);
    assert_eq!(actions.len(), 0);
    assert_eq!(tracker.fired_count(), 1);

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

    let _ = tracker.trigger_at_index(5.0, &runtime);
    assert_eq!(tracker.fired_count(), 1);

    tracker.reset();
    assert_eq!(tracker.fired_count(), 0);

    let actions = tracker.trigger_at_index(5.0, &runtime);
    assert_eq!(actions.len(), 1);
    assert_eq!(tracker.fired_count(), 1);
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
