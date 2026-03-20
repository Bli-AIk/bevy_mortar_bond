//! This crate provides a 'bond' (binding) system for the Mortar dialogue system, integrating it with the Bevy game engine.
//!
//! 本包为 Mortar 对话系统提供“绑钉”（绑定）系统，将其与 Bevy 游戏引擎集成。
//!
//! # ECS Architecture
//!
//! This library follows ECS (Entity Component System) design principles.
//!
//! ## Core Components
//!
//! - [`MortarEventTracker`]: Tracks text events and manages firing state based on index
//!
//! ## Usage Pattern
//!
//! 1. Add [`MortarEventTracker`] to entities with text events
//! 2. Call `trigger_at_index()` with current progress index
//! 3. Handle returned [`MortarEventAction`]s in your game systems
//!
//! # ECS 架构
//!
//! 本库遵循 ECS（实体组件系统）设计理念。
//!
//! ## 核心组件
//!
//! - [`MortarEventTracker`]：按索引跟踪文本事件并管理触发状态
//!
//! ## 使用方式
//!
//! 1. 将 [`MortarEventTracker`] 添加到包含文本事件的实体
//! 2. 使用当前进度索引调用 `trigger_at_index()`
//! 3. 在游戏系统中处理返回的 [`MortarEventAction`]

use bevy::prelude::*;
#[cfg(test)]
use mortar_compiler::Node;

#[macro_use]
mod debug;
mod asset;
mod audio;
mod binder;
mod dialogue;
mod dialogue_state;
mod eval;
mod events;
mod runtime;
mod system;
mod variable_state;

#[cfg(test)]
mod tests;

pub use asset::{MortarAsset, MortarAssetLoader};
pub use audio::MortarAudioSettings;
pub use bevy_mortar_bond_macros::{MortarFunctions, mortar_functions};
pub use binder::{
    MortarBoolean, MortarFunctionRegistry, MortarNumber, MortarString, MortarValue, MortarVoid,
};
pub use dialogue::{
    CachedCondition, MortarDialoguePlugin, MortarDialogueSystemSet, MortarDialogueText,
    MortarDialogueVariables, MortarEventBinding, MortarGameEvent, MortarRunsExecuting,
    MortarTextTarget, evaluate_condition_cached,
};
pub use dialogue_state::{
    DialogueRunDescriptor, DialogueRunItem, DialogueRunKind, DialogueState, TextData,
};
pub use eval::{evaluate_condition, evaluate_if_condition, process_interpolated_text};
pub use events::{MortarDialogueFinished, MortarEvent, MortarEventAction, MortarEventTracker};
pub use runtime::{MortarRegistry, MortarRuntime};
pub use variable_state::{MortarVariableState, MortarVariableValue};

/// Re-export mortar_compiler types for convenience.
///
/// 为方便使用，重新导出 mortar_compiler 类型。
pub use mortar_compiler::Event as MortarTextEvent;

/// Convenient re-exports for common usage.
///
/// 常用类型的便捷重导出。
pub mod prelude {
    pub use crate::{
        MortarAudioSettings, MortarDialoguePlugin, MortarDialogueSystemSet, MortarDialogueText,
        MortarDialogueVariables, MortarEventBinding, MortarFunctionRegistry, MortarGameEvent,
        MortarPlugin, MortarRunsExecuting, MortarTextTarget, MortarValue,
    };
}

/// The main plugin for the mortar 'bond' (bind) system.
///
/// Mortar "绑钉" （绑定）系统的主要插件。
pub struct MortarPlugin;

impl Plugin for MortarPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MortarAsset>()
            .init_asset_loader::<MortarAssetLoader>()
            .init_resource::<MortarRegistry>()
            .init_resource::<MortarRuntime>()
            .add_message::<MortarEvent>()
            .add_message::<MortarDialogueFinished>()
            .add_systems(
                Update,
                (
                    system::process_mortar_events_system,
                    system::check_pending_start_system,
                    system::handle_pending_jump_system,
                )
                    .chain(),
            );
    }
}
