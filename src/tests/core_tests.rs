//! Acts as the root module for `bevy_mortar_bond`'s focused core test suite.
//! It groups the unit tests that cover the runtime's fundamental data structures
//! and exposes a small shared fixture builder so the submodules can reuse the same
//! representative dialogue node.
//!
//! `bevy_mortar_bond` 核心测试集的根模块。它把覆盖运行时基础数据结构的
//! 单元测试组织在一起，并提供一个共享的 fixture 构造函数，方便子模块复用同一个代表性
//! 对话节点。

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
