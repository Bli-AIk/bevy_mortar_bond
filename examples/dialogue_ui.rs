//! Advanced UI demo for `bevy_mortar_bond`.
//!
//! This example now focuses purely on UI + gameplay reactions. All Mortar runtime
//! mechanics (text interpolation, timelines, waits, etc.) are handled by
//! `MortarDialoguePlugin`, so we only configure assets and respond to events.
//!
//! `bevy_mortar_bond` 的高级 UI 演示示例。
//!
//! 该示例专注于 UI 与玩法响应，Mortar 运行时机制（文本插值、时间线、等待等）
//! 均由 `MortarDialoguePlugin` 负责，我们只需配置资源并响应事件。

mod utils;

use bevy::prelude::*;
use bevy_ecs_typewriter::TypewriterPlugin;
use bevy_mortar_bond::{
    MortarDialoguePlugin, MortarEvent, MortarFunctions, MortarGameEvent, MortarNumber,
    MortarPlugin, MortarRegistry, MortarRuntime, MortarString, mortar_functions,
};
use std::time::Duration;
use utils::ui::*;

/// Component that tags the triangle hero sprite.
#[derive(Component)]
struct TriangleSprite;

/// Resource that cycles through bundled `.mortar` files with a button.
#[derive(Resource)]
struct DialogueFiles {
    files: Vec<String>,
    current_index: usize,
}

impl Default for DialogueFiles {
    fn default() -> Self {
        Self {
            files: vec![
                "pub.mortar".into(),
                "demo.mortar".into(),
                "simple.mortar".into(),
                "basic.mortar".into(),
                "control_flow.mortar".into(),
                "performance_system.mortar".into(),
                "branch_interpolation.mortar".into(),
                "enum_branch.mortar".into(),
                "master_test.mortar".into(),
            ],
            current_index: 0,
        }
    }
}

impl DialogueFiles {
    fn current(&self) -> &str {
        &self.files[self.current_index]
    }

    fn next(&mut self) {
        self.current_index = (self.current_index + 1) % self.files.len();
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MortarPlugin,
            MortarDialoguePlugin,
            TypewriterPlugin,
            DialogueUiPlugin,
        ))
        .init_resource::<DialogueFiles>()
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_triangle_sprite,
                setup_mortar_functions,
                load_initial_dialogue,
            )
                .chain(),
        )
        .add_systems(Update, (handle_game_events, update_rotate_animation))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_triangle_sprite(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let triangle = Mesh::from(Triangle2d::new(
        Vec2::new(0.0, 30.0),
        Vec2::new(-25.0, -15.0),
        Vec2::new(25.0, -15.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(triangle)),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.8, 0.9))),
        Transform::from_xyz(0.0, 300.0, 0.0),
        TriangleSprite,
    ));
}

#[derive(MortarFunctions)]
struct GameFunctions;

#[mortar_functions]
impl GameFunctions {
    fn get_name() -> String {
        info!("Example: Getting player name");
        "U-S-E-R".into()
    }

    fn get_exclamation(count: MortarNumber) -> String {
        let n = count.as_usize();
        info!("Example: exclamation count {}", n);
        "！".repeat(n)
    }

    fn create_message(verb: MortarString, obj: MortarString, level: MortarNumber) -> String {
        let verb = verb.as_str();
        let obj = obj.as_str();
        let level = level.as_usize();
        info!(
            "Example: Creating message verb={} obj={} level={}",
            verb, obj, level
        );
        format!("{}{}{}", verb, obj, "!".repeat(level))
    }

    fn play_sound(file_name: MortarString) -> MortarString {
        info!("Example: Requesting sound {}", file_name.as_str());
        file_name
    }

    fn has_map() -> bool {
        true
    }

    fn has_backpack() -> bool {
        false
    }

    fn set_animation(anim_name: MortarString) {
        info!("Example: Requesting animation {}", anim_name.as_str());
    }

    fn set_color(color: MortarString) {
        info!("Example: Requesting color {}", color.as_str());
    }
}

fn setup_mortar_functions(mut runtime: ResMut<MortarRuntime>) {
    GameFunctions::bind_functions(&mut runtime.functions);
}

fn load_initial_dialogue(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    dialogue_files: Res<DialogueFiles>,
) {
    let path = dialogue_files.current().to_string();
    info!("Example: Start loading files: {}", &path);
    let handle = asset_server.load(&path);
    registry.register(path.clone(), handle);

    const START_NODE: &str = "Start";
    info!("Example: Send StartNode event: {} / {}", &path, START_NODE);
    events.write(MortarEvent::StartNode {
        path,
        node: START_NODE.to_string(),
    });
}

/// Read `MortarGameEvent`s and convert them into Bevy gameplay actions.
fn handle_game_events(
    mut events: MessageReader<MortarGameEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut triangle_transforms: Query<(Entity, &mut Transform), With<TriangleSprite>>,
    triangle_materials: Query<&MeshMaterial2d<ColorMaterial>, With<TriangleSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in events.read() {
        match event.name.as_str() {
            "set_animation" => {
                if let Some(anim_name) = event.args.first()
                    && let Ok((entity, mut transform)) = triangle_transforms.single_mut()
                {
                    match anim_name.as_str() {
                        "wave" => {
                            commands.entity(entity).insert(RotateAnimation {
                                timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
                                start_rotation: transform.rotation.to_euler(EulerRot::ZXY).0,
                            });
                        }
                        "left" => transform.translation.x = -50.0,
                        "right" => transform.translation.x = 50.0,
                        _ => {}
                    }
                }
            }
            "set_color" => {
                if let Some(color_hex) = event.args.first()
                    && let Some(color) = parse_hex_color(color_hex)
                    && let Ok(material_handle) = triangle_materials.single()
                    && let Some(material) = materials.get_mut(&material_handle.0)
                {
                    material.color = color;
                    info!("Example: Triangle color changed to {}", color_hex);
                }
            }
            "play_sound" => {
                if let Some(path) = event.args.first() {
                    let audio_source = asset_server.load::<AudioSource>(path.clone());
                    commands.spawn(AudioPlayer::new(audio_source));
                }
            }
            _ => {
                info!(
                    "Example: Received custom event {} {:?}",
                    event.name, event.args
                );
            }
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::srgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    ))
}

#[derive(Component)]
struct RotateAnimation {
    timer: Timer,
    start_rotation: f32,
}

fn update_rotate_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut RotateAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        let progress = anim.timer.fraction();
        let angle = anim.start_rotation + progress * std::f32::consts::TAU;
        transform.rotation = Quat::from_rotation_z(angle);

        if anim.timer.just_finished() {
            commands.entity(entity).remove::<RotateAnimation>();
            transform.rotation = Quat::from_rotation_z(0.0);
        }
    }
}
