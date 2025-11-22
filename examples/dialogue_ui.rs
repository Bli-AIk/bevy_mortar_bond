//! A simple dialogue UI example for the `bevy_mortar_bond` crate.
//!
//! This example demonstrates how to bind Mortar dialogue system to a Bevy UI.
//! UI components are separated into the utils module for clarity.
//!
//! `bevy_mortar_bond` 包的一个简单对话 UI 示例。
//!
//! 此示例演示如何将 Mortar 对话系统绑定到 Bevy UI。
//! UI 组件已分离到 utils 模块中以提高清晰度。

mod utils;

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterPlugin, TypewriterState};
use bevy_mortar_bond::{
    MortarEvent, MortarEventAction, MortarEventTracker, MortarFunctions, MortarNumber,
    MortarPlugin, MortarRegistry, MortarRuntime, MortarString,
};
use std::time::Duration;
use utils::ui::*;

/// Component to mark entities that need animation
///
/// 标记需要动画的实体的组件
#[derive(Component)]
struct PendingAnimation(String);

/// Component to mark entities that need color change
///
/// 标记需要改变颜色的实体的组件
#[derive(Component)]
struct PendingColorChange(String);

/// Marker component for the triangle sprite
///
/// 三角形精灵标记组件
#[derive(Component)]
struct TriangleSprite;

/// Resource to track the current dialogue file and available files.
///
/// 资源：跟踪当前对话文件和可用文件。
#[derive(Resource)]
struct DialogueFiles {
    files: Vec<String>,
    current_index: usize,
}

impl Default for DialogueFiles {
    fn default() -> Self {
        Self {
            files: vec![
                "simple.mortar".to_string(),
                "demo.mortar".to_string(),
                "basic.mortar".to_string(),
                "branch_interpolation.mortar".to_string(),
                "control_flow.mortar".to_string(),
                "performance_system.mortar".to_string(),
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
        .add_plugins((DefaultPlugins, MortarPlugin, TypewriterPlugin))
        .init_resource::<DialogueFiles>()
        .add_systems(
            Startup,
            (
                setup,
                setup_triangle_sprite,
                setup_mortar_functions,
                load_initial_dialogue,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                button_interaction_system,
                handle_continue_button,
                handle_choice_buttons,
                handle_reload_button,
                handle_switch_file_button,
                update_dialogue_text_with_typewriter,
                manage_choice_buttons,
                update_button_states,
                trigger_typewriter_events,
                // Pure ECS systems handling game effects
                apply_pending_animations,
                apply_pending_colors,
                update_rotate_animation,
            ),
        )
        .run();
}

/// Sets up the camera and UI.
///
/// 设置相机和 UI。
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let font = asset_server.load("Unifont.otf");
    setup_dialogue_ui(&mut commands, font);
}

/// Sets up the triangle sprite at the top of the screen.
///
/// 在屏幕顶部设置三角形精灵。
fn setup_triangle_sprite(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create a triangle mesh
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

#[bevy_mortar_bond::mortar_functions]
impl GameFunctions {
    fn get_name() -> String {
        info!("Getting player name");
        "U-S-E-R".to_string()
    }

    fn get_exclamation(count: MortarNumber) -> String {
        let n = count.as_usize();
        info!("Getting exclamation with count: {}", n);
        "！".repeat(n)
    }

    fn create_message(verb: MortarString, obj: MortarString, level: MortarNumber) -> String {
        let v = verb.as_str();
        let o = obj.as_str();
        let l = level.as_usize();
        info!("Creating message: verb={}, obj={}, level={}", v, o, l);
        format!("{}{}{}", v, o, "!".repeat(l))
    }

    fn play_sound(file_name: MortarString) {
        info!("Playing sound: {}", file_name.as_str());
        // Note: Audio playback disabled due to backend configuration
    }

    fn set_animation(anim_name: MortarString) {
        info!("Queuing animation: {}", anim_name.as_str());
        // Animation will be applied by the system that processes events
    }

    fn set_color(color: MortarString) {
        info!("Queuing color change: {}", color.as_str());
        // Color will be applied by the system that processes events
    }
}

fn setup_mortar_functions(mut runtime: ResMut<MortarRuntime>) {
    GameFunctions::bind_functions(&mut runtime.functions);
}

/// Loads the initial dialogue file and starts the first node.
///
/// 加载初始对话文件并启动第一个节点。
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

/// Handles clicks on the "Continue" button.
///
/// 处理"继续"按钮的点击事件。
fn handle_continue_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut events: MessageWriter<MortarEvent>,
    runtime: Res<MortarRuntime>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed
            && let Some(state) = &runtime.active_dialogue
        {
            // If a choice is selected, confirm it
            if state.selected_choice.is_some() {
                info!("Example: Confirming choice selection");
                events.write(MortarEvent::ConfirmChoice);
            } else {
                // Otherwise, advance text
                events.write(MortarEvent::NextText);
                if !state.has_next_text() {
                    info!(
                        "Example: Reached end of text in node '{}'",
                        state.current_node
                    );
                }
            }
        }
    }
}

/// Handles clicks on choice buttons.
///
/// 处理选项按钮的点击事件。
fn handle_choice_buttons(
    choice_query: Query<(&Interaction, &ChoiceButton), Changed<Interaction>>,
    mut events: MessageWriter<MortarEvent>,
) {
    for (interaction, choice_button) in &choice_query {
        if *interaction == Interaction::Pressed {
            info!("Example: Choice button {} pressed", choice_button.index);
            events.write(MortarEvent::SelectChoice {
                index: choice_button.index,
            });
        }
    }
}

/// Dynamically creates and updates choice buttons based on dialogue state.
///
/// 根据对话状态动态创建和更新选项按钮。
fn manage_choice_buttons(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    container_query: Query<Entity, With<ChoiceContainer>>,
    button_query: Query<Entity, With<ChoiceButton>>,
    asset_server: Res<AssetServer>,
) {
    if !runtime.is_changed() {
        return;
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    // Clear existing buttons
    for entity in button_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create new buttons if we have choices
    if let Some(state) = &runtime.active_dialogue
        && let Some(choices) = state.get_choices()
        && !state.has_next_text()
    {
        let font = asset_server.load("Unifont.otf");

        for (index, choice) in choices.iter().enumerate() {
            let is_selected = state.selected_choice == Some(index);

            let (bg_color, border_color, text_color) = if is_selected {
                // Selected style
                (
                    Color::srgb(0.3, 0.4, 0.6),
                    Color::srgb(0.5, 0.7, 0.9),
                    Color::srgb(1.0, 1.0, 1.0),
                )
            } else {
                // Normal style
                (
                    Color::srgb(0.2, 0.25, 0.35),
                    Color::srgb(0.4, 0.5, 0.65),
                    Color::srgb(0.85, 0.85, 0.85),
                )
            };

            commands.entity(container).with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        BorderColor::all(border_color),
                        ChoiceButton { index },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(&choice.text),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });
            });
        }
    }
}

/// Updates the continue button state.
///
/// 更新继续按钮状态。
fn update_button_states(
    runtime: Res<MortarRuntime>,
    mut continue_query: Query<(&mut Text, &mut Visibility), With<ContinueButton>>,
) {
    if !runtime.is_changed() {
        return;
    }

    for (mut text, mut visibility) in continue_query.iter_mut() {
        if let Some(state) = &runtime.active_dialogue {
            if state.has_choices() && !state.has_next_text() {
                // Has choices - show continue button only if choice is selected
                if state.selected_choice.is_some() {
                    *visibility = Visibility::Visible;
                    **text = "确认选择".to_string();
                } else {
                    *visibility = Visibility::Hidden;
                }
            } else {
                // No choices or has more text
                *visibility = Visibility::Visible;
                **text = "继续".to_string();
            }
        } else {
            *visibility = Visibility::Visible;
            **text = "继续".to_string();
        }
    }
}

/// Handles clicks on the "Reload" button.
///
/// 处理"重载"按钮的点击事件。
fn handle_reload_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReloadButton>)>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    dialogue_files: Res<DialogueFiles>,
    runtime: Res<MortarRuntime>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let path = dialogue_files.current().to_string();
            info!("Example: Reload file: {}", &path);

            // Stop current dialogue
            events.write(MortarEvent::StopDialogue);

            // Reload asset
            let handle = asset_server.load(&path);
            registry.register(path.clone(), handle);

            // Restart from the current node or Start
            let start_node = runtime
                .active_dialogue
                .as_ref()
                .map(|state| state.current_node.clone())
                .unwrap_or_else(|| "Start".to_string());

            info!("Example: 重新启动节点: {} / {}", &path, &start_node);
            events.write(MortarEvent::StartNode {
                path,
                node: start_node,
            });
        }
    }
}

/// Handles clicks on the "Switch File" button.
///
/// 处理"切换文件"按钮的点击事件。
fn handle_switch_file_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SwitchFileButton>)>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    mut dialogue_files: ResMut<DialogueFiles>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Stop current dialogue
            events.write(MortarEvent::StopDialogue);

            // Switch to next file
            dialogue_files.next();
            let path = dialogue_files.current().to_string();
            info!("Example: Switch to file: {}", &path);

            // Load new file
            let handle = asset_server.load(&path);
            registry.register(path.clone(), handle);

            // Start from the beginning
            const START_NODE: &str = "Start";
            info!("Example: Start a new file node: {} / {}", &path, START_NODE);
            events.write(MortarEvent::StartNode {
                path,
                node: START_NODE.to_string(),
            });
        }
    }
}

/// Updates dialogue text with typewriter effect
///
/// 使用打字机效果更新对话文本
fn update_dialogue_text_with_typewriter(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    mut dialogue_query: Query<(Entity, &mut Text), With<DialogueText>>,
    typewriter_query: Query<&Typewriter, With<DialogueText>>,
    mut last_key: Local<Option<(String, String, usize)>>,
) {
    if !runtime.is_changed() {
        return;
    }

    for (entity, mut text) in &mut dialogue_query {
        if let Some(state) = &runtime.active_dialogue {
            let current_key = (
                state.mortar_path.clone(),
                state.current_node.clone(),
                state.text_index,
            );

            let should_process = last_key.as_ref() != Some(&current_key);

            if should_process && let Some(text_data) = state.current_text_data() {
                let processed_text =
                    bevy_mortar_bond::process_interpolated_text(text_data, &runtime.functions);

                info!("Starting typewriter for: {}", processed_text);

                // Remove old typewriter if exists
                if typewriter_query.get(entity).is_ok() {
                    commands.entity(entity).remove::<Typewriter>();
                    commands.entity(entity).remove::<MortarEventTracker>();
                }

                // Create new typewriter - only for dialogue text
                let mut typewriter = Typewriter::new(&processed_text, 0.05);
                typewriter.play();
                commands.entity(entity).insert(typewriter);

                // Add dialogue events tracking using library component
                if let Some(events) = &text_data.events {
                    commands
                        .entity(entity)
                        .insert(MortarEventTracker::new(events.clone()));
                }

                *last_key = Some(current_key);
            }

            // Update text: static header + typewriter dialogue
            if let Ok(typewriter) = typewriter_query.get(entity) {
                let header = format!("[{} / {}]\n\n", state.mortar_path, state.current_node);
                **text = format!("{}{}", header, typewriter.current_text);
            }
        } else {
            **text = "等待加载对话...".to_string();
            *last_key = None;
        }
    }
}

/// Triggers dialogue events at specific typewriter indices
///
/// 在特定打字机索引处触发对话事件
fn trigger_typewriter_events(
    mut query: Query<(Entity, &Typewriter, &mut MortarEventTracker)>,
    runtime: Res<MortarRuntime>,
    mut commands: Commands,
) {
    for (entity, typewriter, mut tracker) in &mut query {
        if typewriter.state != TypewriterState::Playing {
            continue;
        }

        // Use library's trigger_at_index API - clean and simple
        let actions = tracker.trigger_at_index(typewriter.current_char_index, &runtime);

        // Handle game-specific actions
        for action in actions {
            handle_mortar_action(entity, action, &mut commands);
        }
    }
}

/// Handles a mortar event action by dispatching to appropriate game systems
///
/// 处理 mortar 事件动作，分发到适当的游戏系统
fn handle_mortar_action(entity: Entity, action: MortarEventAction, commands: &mut Commands) {
    match action.action_type.as_str() {
        "set_animation" => {
            if let Some(anim_name) = action.args.first() {
                commands
                    .entity(entity)
                    .insert(PendingAnimation(anim_name.clone()));
            }
        }
        "set_color" => {
            if let Some(color) = action.args.first() {
                commands
                    .entity(entity)
                    .insert(PendingColorChange(color.clone()));
            }
        }
        _ => {}
    }
}

/// Apply pending animations to triangle sprite
///
/// 将待处理的动画应用到三角形精灵
fn apply_pending_animations(
    mut commands: Commands,
    query: Query<(Entity, &PendingAnimation)>,
    triangle_query: Query<Entity, With<TriangleSprite>>,
) {
    for (entity, pending) in &query {
        if pending.0 == "wave" {
            for triangle_entity in triangle_query.iter() {
                commands.entity(triangle_entity).insert(RotateAnimation {
                    timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
                    start_rotation: 0.0,
                });
            }
        }
        // Remove the marker component after processing
        commands.entity(entity).remove::<PendingAnimation>();
    }
}

/// Apply pending color changes to triangle sprite
///
/// 将待处理的颜色变化应用到三角形精灵
fn apply_pending_colors(
    mut commands: Commands,
    query: Query<(Entity, &PendingColorChange)>,
    triangle_query: Query<&MeshMaterial2d<ColorMaterial>, With<TriangleSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, pending) in &query {
        if let Some(parsed_color) = parse_hex_color(&pending.0) {
            for material_handle in triangle_query.iter() {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.color = parsed_color;
                    info!("Triangle color changed to {}", pending.0);
                }
            }
        }
        // Remove the marker component after processing
        commands.entity(entity).remove::<PendingColorChange>();
    }
}

/// Process queued game commands from mortar functions
///
/// Component for rotation animation
#[derive(Component)]
struct RotateAnimation {
    timer: Timer,
    start_rotation: f32,
}

/// System to update rotation animations
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

        if anim.timer.is_finished() {
            commands.entity(entity).remove::<RotateAnimation>();
            transform.rotation = Quat::from_rotation_z(0.0);
        }
    }
}

/// Parse hex color string like "#FF6B6B"
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
