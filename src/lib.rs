use bevy::prelude::*;
use asset::{MortarAsset, MortarAssetLoader};

mod asset;

/// The main plugin for the mortar 'bond' (bind) system.
///
/// Mortar "绑钉" （绑定）系统的主要插件。
pub struct MortarPlugin;

impl Plugin for MortarPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MortarAsset>()
            .init_asset_loader::<MortarAssetLoader>();
    }
}

