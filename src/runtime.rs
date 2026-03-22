use bevy::prelude::*;
use std::collections::HashMap;

/// A global registry for Mortar assets, managing multiple mortar files.
///
/// 全局 Mortar 资源注册表，管理多个 mortar 文件。
#[derive(Resource, Default)]
pub struct MortarRegistry {
    assets: HashMap<String, Handle<crate::MortarAsset>>,
}

impl MortarRegistry {
    /// Registers a mortar asset, using its path as an identifier.
    ///
    /// 注册一个 mortar 资源，使用路径名作为标识符。
    pub fn register(&mut self, path: impl Into<String>, handle: Handle<crate::MortarAsset>) {
        self.assets.insert(path.into(), handle);
    }

    /// Gets the handle for a registered asset.
    ///
    /// 获取已注册的资源句柄。
    pub fn get(&self, path: &str) -> Option<&Handle<crate::MortarAsset>> {
        self.assets.get(path)
    }
}

/// The runtime state for the Mortar system.
/// Now supports multiple concurrent dialogue controllers.
///
/// Mortar 运行时状态。
/// 现在支持多个并发对话控制器。
#[derive(Resource)]
pub struct MortarRuntime {
    /// Active dialogue states keyed by controller entity.
    pub active_dialogues: HashMap<Entity, crate::dialogue_state::DialogueState>,
    /// The "primary" dialogue entity - receives input by default.
    pub primary_dialogue: Option<Entity>,
    /// Pending start requests keyed by controller entity (path, node).
    pub pending_starts: HashMap<Entity, (String, String)>,
    /// Pending jump requests keyed by controller entity (path, node).
    pub pending_jumps: HashMap<Entity, (String, String)>,
    /// The function registry for calling Mortar functions.
    pub functions: crate::MortarFunctionRegistry,
}

impl MortarRuntime {
    pub fn get_dialogue(&self, entity: Entity) -> Option<&crate::dialogue_state::DialogueState> {
        self.active_dialogues.get(&entity)
    }

    pub fn get_dialogue_mut(
        &mut self,
        entity: Entity,
    ) -> Option<&mut crate::dialogue_state::DialogueState> {
        self.active_dialogues.get_mut(&entity)
    }

    pub fn primary_dialogue_state(&self) -> Option<&crate::dialogue_state::DialogueState> {
        self.primary_dialogue
            .and_then(|entity| self.active_dialogues.get(&entity))
    }

    pub fn primary_dialogue_state_mut(
        &mut self,
    ) -> Option<&mut crate::dialogue_state::DialogueState> {
        self.primary_dialogue
            .and_then(|entity| self.active_dialogues.get_mut(&entity))
    }

    pub fn primary_dialogue(&self) -> Option<&crate::dialogue_state::DialogueState> {
        self.primary_dialogue_state()
    }

    pub fn primary_dialogue_mut(&mut self) -> Option<&mut crate::dialogue_state::DialogueState> {
        self.primary_dialogue_state_mut()
    }

    pub fn has_active_dialogues(&self) -> bool {
        !self.active_dialogues.is_empty()
    }

    pub fn active_dialogue_count(&self) -> usize {
        self.active_dialogues.len()
    }
}

impl Default for MortarRuntime {
    fn default() -> Self {
        Self {
            active_dialogues: HashMap::new(),
            primary_dialogue: None,
            pending_starts: HashMap::new(),
            pending_jumps: HashMap::new(),
            functions: crate::MortarFunctionRegistry::new(),
        }
    }
}
