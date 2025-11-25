//! Minimal dialogue example that highlights the new `MortarDialoguePlugin`.
//!
//! The library handles text interpolation, conditionals, and run statements,
//! so this file only needs to load a `.mortar` asset, tag a `Text` entity with
//! [`MortarTextTarget`], and listen for [`MortarGameEvent`]s to drive custom logic.

use bevy::prelude::*;
use bevy_mortar_bond::{
    MortarDialoguePlugin, MortarEvent, MortarGameEvent, MortarPlugin, MortarRegistry,
    MortarTextTarget,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MortarPlugin, MortarDialoguePlugin))
        .add_systems(Startup, (setup_camera, setup_ui, start_dialogue))
        .add_systems(Update, log_game_events)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("Loading Mortar dialogue..."),
        TextFont {
            font: asset_server.load("Unifont.otf"),
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        MortarTextTarget,
    ));
}

fn start_dialogue(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
) {
    let path = "simple.mortar";
    registry.register(path.to_string(), asset_server.load(path));
    events.write(MortarEvent::StartNode {
        path: path.into(),
        node: "Start".into(),
    });
}

fn log_game_events(mut events: MessageReader<MortarGameEvent>) {
    for event in events.read() {
        info!(
            "Simple example received event: {} {:?}",
            event.name, event.args
        );
    }
}
