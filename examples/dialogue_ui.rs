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
    MortarAsset, MortarEvent, MortarEventAction, MortarEventTracker, MortarFunctions, MortarNumber,
    MortarPlugin, MortarRegistry, MortarRuntime, MortarString, MortarVariableState,
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

/// Component to mark that audio needs to be played
///
/// 标记需要播放音频的组件
#[derive(Component)]
struct PendingAudioPlay(String);

/// Marker component for the triangle sprite
///
/// 三角形精灵标记组件
#[derive(Component)]
struct TriangleSprite;

/// Component to track pending run statements with duration
///
/// 追踪带有duration的待执行run语句
#[derive(Component)]
struct PendingRunExecution {
    timer: Timer,
    remaining_runs: Vec<(String, Option<f64>, bool)>, // (event_name, duration, ignore_duration)
    event_defs: Vec<mortar_compiler::EventDef>,
    timeline_defs: Vec<mortar_compiler::TimelineDef>,
    dialogue_entity: Entity,
}

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
                "master_test.mortar".to_string(),
                "performance_system.mortar".to_string(),
                "branch_interpolation.mortar".to_string(),
                "enum_branch.mortar".to_string(),
                "demo.mortar".to_string(),
                "simple.mortar".to_string(),
                "basic.mortar".to_string(),
                "control_flow.mortar".to_string(),
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

/// Resource to hold the runtime variable state
///
/// 持有运行时变量状态的资源
#[derive(Resource, Default)]
struct RuntimeVariableState {
    state: Option<MortarVariableState>,
}

/// Resource to track if runs are currently executing
///
/// 追踪runs是否正在执行的资源
#[derive(Resource, Default)]
struct RunsExecuting {
    executing: bool,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MortarPlugin, TypewriterPlugin))
        .init_resource::<DialogueFiles>()
        .init_resource::<RuntimeVariableState>()
        .init_resource::<RunsExecuting>()
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
                manage_variable_state,
                process_run_statements_after_text,
                clear_runs_executing_flag,
                update_dialogue_text_with_typewriter,
                manage_choice_buttons,
                update_choice_button_styles,
                update_button_states,
                trigger_typewriter_events,
                apply_pending_animations,
                apply_pending_colors,
                play_pending_audio,
                update_rotate_animation,
                process_pending_run_executions,
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
        info!("Example: Getting player name");
        "U-S-E-R".to_string()
    }

    fn get_exclamation(count: MortarNumber) -> String {
        let n = count.as_usize();
        info!("Example: Getting exclamation with count: {}", n);
        "！".repeat(n)
    }

    fn create_message(verb: MortarString, obj: MortarString, level: MortarNumber) -> String {
        let v = verb.as_str();
        let o = obj.as_str();
        let l = level.as_usize();
        info!(
            "Example: Creating message: verb={}, obj={}, level={}",
            v, o, l
        );
        format!("{}{}{}", v, o, "!".repeat(l))
    }

    fn play_sound(file_name: MortarString) -> MortarString {
        info!("Example: Playing sound: {}", file_name.as_str());
        file_name
    }

    fn has_map() -> bool {
        info!("Example: Checking has_map");
        true
    }

    fn has_backpack() -> bool {
        info!("Example: Checking has_backpack");
        false
    }

    fn set_animation(anim_name: MortarString) {
        info!("Example: Queuing animation: {}", anim_name.as_str());
        // Animation will be applied by the system that processes events
    }

    fn set_color(color: MortarString) {
        info!("Example: Queuing color change: {}", color.as_str());
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
    runs_executing: Res<RunsExecuting>,
) {
    // Don't handle clicks if runs are executing
    if runs_executing.executing {
        return;
    }

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
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    mut last_state: Local<Option<(String, String, Vec<usize>, bool)>>, // (path, node, choice_stack, choices_broken)
) {
    if !runtime.is_changed() {
        return;
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    // Check if choice context has changed
    let current_state = runtime.active_dialogue.as_ref().map(|state| {
        (
            state.mortar_path.clone(),
            state.current_node.clone(),
            state.choice_stack.clone(),
            state.choices_broken,
        )
    });

    // Only recreate buttons if choice context actually changed
    if *last_state == current_state && !button_query.is_empty() {
        return;
    }

    *last_state = current_state;

    // Clear existing buttons
    for entity in button_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create new buttons if we have choices
    // Show choices after the text at choice_position has been displayed
    if let Some(state) = &runtime.active_dialogue
        && let Some(choices) = state.get_choices()
    {
        // Check if we should show choices
        // In the new architecture, choices appear at a specific position in the content array
        // We show them when we've reached or passed that position
        let should_show_choices = !state.has_next_text_before_choice();

        if !should_show_choices {
            return;
        }
        let font = asset_server.load("Unifont.otf");

        // Get function declarations for condition evaluation
        let function_decls = registry
            .get(&state.mortar_path)
            .and_then(|handle| assets.get(handle))
            .map(|asset| asset.data.functions.as_slice())
            .unwrap_or(&[]);

        for (index, choice) in choices.iter().enumerate() {
            let is_selected = state.selected_choice == Some(index);

            // Evaluate condition if present
            let is_enabled = choice
                .condition
                .as_ref()
                .map(|cond| {
                    bevy_mortar_bond::evaluate_condition(cond, &runtime.functions, function_decls)
                })
                .unwrap_or(true); // No condition means always enabled

            let (bg_color, border_color, text_color) = if !is_enabled {
                // Disabled style (grayed out)
                (
                    Color::srgb(0.15, 0.15, 0.15),
                    Color::srgb(0.25, 0.25, 0.25),
                    Color::srgb(0.4, 0.4, 0.4),
                )
            } else if is_selected {
                // Selected style - bright and obvious
                (
                    Color::srgb(0.4, 0.6, 0.2),
                    Color::srgb(0.6, 0.9, 0.3),
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

/// Updates choice button styles based on selection state.
///
/// 根据选择状态更新选项按钮样式。
fn update_choice_button_styles(
    runtime: Res<MortarRuntime>,
    mut button_query: Query<(&ChoiceButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    if !runtime.is_changed() {
        return;
    }

    let Some(state) = &runtime.active_dialogue else {
        return;
    };

    for (choice_button, mut bg_color, mut border_color) in button_query.iter_mut() {
        let is_selected = state.selected_choice == Some(choice_button.index);

        if is_selected {
            // Selected style - bright green
            *bg_color = BackgroundColor(Color::srgb(0.4, 0.6, 0.2));
            *border_color = BorderColor::all(Color::srgb(0.6, 0.9, 0.3));
        } else {
            // Normal style
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
            *border_color = BorderColor::all(Color::srgb(0.4, 0.5, 0.65));
        }
    }
}

/// Updates the continue button state.
///
/// 更新继续按钮状态。
fn update_button_states(
    runtime: Res<MortarRuntime>,
    mut continue_query: Query<
        (
            &mut Text,
            &mut Visibility,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<ContinueButton>,
    >,
    runs_executing: Res<RunsExecuting>,
) {
    if !runtime.is_changed() && !runs_executing.is_changed() {
        return;
    }

    for (mut text, mut visibility, mut bg_color, mut border_color) in continue_query.iter_mut() {
        // Apply disabled style if runs are being executed
        if runs_executing.executing {
            *visibility = Visibility::Visible;
            **text = "执行中...".to_string();
            // Disabled style - grayed out
            *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            *border_color = BorderColor::all(Color::srgb(0.25, 0.25, 0.25));
            continue;
        }

        // Normal enabled style
        *bg_color = BackgroundColor(Color::srgb(0.2, 0.4, 0.6));
        *border_color = BorderColor::all(Color::srgb(0.4, 0.6, 0.8));

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

/// Manages variable state lifecycle
///
/// 管理变量状态生命周期
fn manage_variable_state(
    runtime: Res<MortarRuntime>,
    mut var_state_res: ResMut<RuntimeVariableState>,
    mut last_mortar_path: Local<Option<String>>,
) {
    if let Some(state) = &runtime.active_dialogue {
        let current_path = state.mortar_path.clone();

        // Only reset variable state when mortar file changes, not when node changes
        if last_mortar_path.as_ref() != Some(&current_path) {
            info!("Example: Resetting variable state for new mortar file");
            var_state_res.state = None;
            *last_mortar_path = Some(current_path);
        }
    } else if last_mortar_path.is_some() {
        // Dialogue ended, clear state
        var_state_res.state = None;
        *last_mortar_path = None;
    }
}

/// Process run statements after the current text (when user advances)
///
/// 处理当前文本之后的run语句（当用户前进时）
fn process_run_statements_after_text(
    mut commands: Commands,
    mut runtime: ResMut<MortarRuntime>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    mut dialogue_text_query: Query<&mut Text, With<DialogueText>>,
    dialogue_entity_query: Query<Entity, With<DialogueText>>,
    mut runs_executing: ResMut<RunsExecuting>,
) {
    if !runtime.is_changed() {
        return;
    }

    let Some(state) = &mut runtime.active_dialogue else {
        return;
    };

    // In the new architecture, runs appear as content items between or after texts
    // We need to find the content position after the current text and collect any consecutive runs
    
    // Get the current text's content position
    let Some(current_text_content_idx) = state.current_text_content_index() else {
        return; // No valid text position
    };

    // Find consecutive run items after the current text position
    let mut run_sequence = Vec::new();
    let mut content_indices_to_mark = Vec::new();
    let start_search_idx = current_text_content_idx + 1;

    for (idx, content_value) in state.node_data().content.iter().enumerate().skip(start_search_idx) {
        if state.executed_content_indices.contains(&idx) {
            continue;
        }

        if let Some(type_str) = content_value.get("type").and_then(|v| v.as_str()) {
            match type_str {
                "run_event" => {
                    if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                        let ignore_duration = content_value
                            .get("ignore_duration")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        run_sequence.push((name.to_string(), None::<f64>, ignore_duration));
                        content_indices_to_mark.push(idx);
                    }
                }
                "run_timeline" => {
                    if let Some(name) = content_value.get("name").and_then(|v| v.as_str()) {
                        run_sequence.push((name.to_string(), None::<f64>, false));
                        content_indices_to_mark.push(idx);
                    }
                }
                _ => {
                    // Stop at non-run items (text, choice, etc.)
                    break;
                }
            }
        } else {
            break;
        }
    }

    if run_sequence.is_empty() {
        // Clear pending position even if no runs
        state.pending_run_position = None;
        return;
    }

    // Get asset data
    let Some(handle) = registry.get(&state.mortar_path) else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };

    let event_defs = &asset.data.events;
    let timeline_defs = &asset.data.timelines;

    // Fill in durations from event definitions
    let run_sequence_with_durations: Vec<(String, Option<f64>, bool)> = run_sequence
        .iter()
        .map(|(name, _, ignore_duration)| {
            let duration = event_defs
                .iter()
                .find(|e| e.name == *name)
                .and_then(|e| e.duration);
            (name.clone(), duration, *ignore_duration)
        })
        .collect();

    info!(
        "Example: Executing {} consecutive run(s) after current text",
        run_sequence_with_durations.len()
    );

    // Set flag to indicate runs are executing
    runs_executing.executing = true;

    // Clear the dialogue text during execution
    if let Ok(mut text) = dialogue_text_query.single_mut() {
        **text = String::new();
        info!("Example: Cleared dialogue text during run execution");
    }

    // Get the dialogue entity to pass to execute functions
    let Ok(dialogue_entity) = dialogue_entity_query.single() else {
        return;
    };

    // If there are multiple consecutive runs, treat them like a timeline sequence with durations
    if run_sequence_with_durations.len() > 1 {
        start_timeline_execution(
            run_sequence_with_durations,
            event_defs.to_vec(),
            timeline_defs.to_vec(),
            dialogue_entity,
            &mut commands,
        );
    } else if let Some((event_name, _, _)) = run_sequence_with_durations.first() {
        // Single run, execute immediately (no duration wait)
        execute_run_by_name(
            event_name,
            event_defs,
            timeline_defs,
            &mut commands,
            dialogue_entity,
        );
        // No pending execution for single run without duration
        runs_executing.executing = false;
    }

    // Mark content items as executed and clear pending position
    if let Some(state) = &mut runtime.active_dialogue {
        for idx in content_indices_to_mark {
            state.mark_content_executed(idx);
        }
        state.pending_run_position = None;
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
    mut skip_next_conditional: Local<bool>,
    registry: Res<MortarRegistry>,
    assets: Res<Assets<MortarAsset>>,
    mut events: MessageWriter<MortarEvent>,
    mut var_state_res: ResMut<RuntimeVariableState>,
    runs_executing: Res<RunsExecuting>,
) {
    // Don't update text if runs are being executed
    if runs_executing.executing {
        return;
    }

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
                // Get asset data
                let asset_data = registry
                    .get(&state.mortar_path)
                    .and_then(|handle| assets.get(handle))
                    .map(|asset| &asset.data);

                let function_decls = asset_data
                    .map(|data| data.functions.as_slice())
                    .unwrap_or(&[]);

                // Get or initialize variable state
                if var_state_res.state.is_none() {
                    let variables = asset_data
                        .map(|data| data.variables.as_slice())
                        .unwrap_or(&[]);
                    let enums = asset_data.map(|data| data.enums.as_slice()).unwrap_or(&[]);
                    var_state_res.state =
                        Some(MortarVariableState::from_variables(variables, enums));
                }

                let variable_state = var_state_res.state.as_mut().unwrap();

                // Check if we should skip this conditional text (it's the else branch of a previous if)
                if *skip_next_conditional && text_data.condition.is_some() {
                    info!("Example: Skipping else branch, auto-advancing");
                    *skip_next_conditional = false;
                    *last_key = Some(current_key);
                    events.write(MortarEvent::NextText);
                    continue;
                }

                // Check if condition is satisfied FIRST
                if let Some(condition) = &text_data.condition
                    && !bevy_mortar_bond::evaluate_if_condition(
                        condition,
                        &runtime.functions,
                        &variable_state,
                    )
                {
                    info!("Example: Condition not satisfied, auto-advancing to next text");
                    *skip_next_conditional = false;
                    // Skip this text and automatically advance to the next one
                    *last_key = Some(current_key);
                    events.write(MortarEvent::NextText);
                    continue;
                }

                // Execute pre_statements only if condition is satisfied (or no condition)
                let has_statements = !text_data.pre_statements.is_empty();
                for stmt in &text_data.pre_statements {
                    if stmt.stmt_type == "assignment" {
                        if let (Some(var_name), Some(value)) = (&stmt.var_name, &stmt.value) {
                            info!("Example: Executing assignment: {} = {}", var_name, value);
                            variable_state.execute_assignment(var_name, value);
                        }
                    }
                }

                let processed_text = bevy_mortar_bond::process_interpolated_text(
                    text_data,
                    &runtime.functions,
                    function_decls,
                    variable_state,
                );

                // Skip empty texts (they were just for executing statements)
                if processed_text.is_empty() {
                    info!("Example: Empty text after statements, auto-advancing");
                    // If this was a conditional statement execution, skip the next conditional (else branch)
                    if has_statements && text_data.condition.is_some() {
                        *skip_next_conditional = true;
                    }
                    *last_key = Some(current_key);
                    events.write(MortarEvent::NextText);
                    continue;
                }

                // Clear skip flag for non-conditional or non-empty texts
                *skip_next_conditional = false;

                info!("Example: Starting typewriter for: {}", processed_text);

                // Remove old typewriter if exists
                if typewriter_query.get(entity).is_ok() {
                    commands.entity(entity).remove::<Typewriter>();
                    commands.entity(entity).remove::<MortarEventTracker>();
                }

                // Create new typewriter - only for dialogue text
                let mut typewriter = Typewriter::new(&processed_text, 0.05);
                typewriter.play();
                commands.entity(entity).insert(typewriter);

                // Collect events from text_data.events and run statements with index_override
                let mut all_events = Vec::new();

                // If we have interpolated text, we need to map indices from the original text to the rendered text
                if let Some(parts) = &text_data.interpolated_parts {
                    if let Some(asset_data) = asset_data {
                        // Build index mapping: original_index -> rendered_index
                        let mut original_pos = 0.0;
                        let mut rendered_pos = 0.0;
                        let mut index_map = std::collections::HashMap::new();

                        for part in parts {
                            if part.part_type == "text" {
                                // For each character in the text part, map original to rendered position
                                for _ in 0..part.content.chars().count() {
                                    index_map.insert(original_pos as usize, rendered_pos);
                                    original_pos += 1.0;
                                    rendered_pos += 1.0;
                                }
                            } else if part.part_type == "placeholder" {
                                let var_name = part.content.trim_matches(|c| c == '{' || c == '}');

                                // Add branch variable events at the current rendered position
                                if let Some(branch_events) = variable_state
                                    .get_branch_events(var_name, &asset_data.variables)
                                {
                                    for mut event in branch_events {
                                        event.index += rendered_pos;
                                        info!(
                                            "Example: Added branch event from '{}' at rendered pos {} -> index {}",
                                            var_name, rendered_pos, event.index
                                        );
                                        all_events.push(event);
                                    }
                                }

                                // Calculate the length of the replaced text
                                if let Some(branch_text) = variable_state.get_branch_text(var_name)
                                {
                                    rendered_pos += branch_text.chars().count() as f64;
                                } else if let Some(value) = variable_state.get(var_name) {
                                    rendered_pos +=
                                        value.to_display_string().chars().count() as f64;
                                }
                            }
                        }

                        // Map the final position (for events at the end of text)
                        index_map.insert(original_pos as usize, rendered_pos);

                        // Now adjust text_data.events using the index map
                        if let Some(text_events) = &text_data.events {
                            for event in text_events {
                                let mut adjusted_event = event.clone();

                                // Resolve index_variable if present
                                if let Some(var_name) = &adjusted_event.index_variable {
                                    if let Some(value) = variable_state.get(var_name) {
                                        if let bevy_mortar_bond::MortarVariableValue::Number(n) =
                                            value
                                        {
                                            adjusted_event.index = *n;
                                            info!(
                                                "Example: Resolved index_variable '{}' to {}",
                                                var_name, n
                                            );
                                        }
                                    }
                                }

                                // Map the index from original to rendered position
                                let original_index = adjusted_event.index as usize;
                                if let Some(&rendered_index) = index_map.get(&original_index) {
                                    adjusted_event.index = rendered_index;
                                    info!(
                                        "Example: Mapped text event index {} -> {} (interpolated text)",
                                        original_index, rendered_index
                                    );
                                }

                                all_events.push(adjusted_event);
                            }
                        }
                    }
                } else {
                    // No interpolation, just use events as-is
                    all_events = text_data.events.clone().unwrap_or_default();

                    // Resolve index_variable for events that have it
                    for event in &mut all_events {
                        if let Some(var_name) = &event.index_variable {
                            if let Some(value) = variable_state.get(var_name) {
                                if let bevy_mortar_bond::MortarVariableValue::Number(n) = value {
                                    event.index = *n;
                                    info!(
                                        "Example: Resolved index_variable '{}' to {}",
                                        var_name, n
                                    );
                                }
                            }
                        }
                    }
                }

                // Add events from run_event items with index_override at current text position
                // In the new architecture, run events with index_override appear in the content array
                // We need to check the content item at the current text's position
                if let Some(asset_data) = asset_data
                    && let Some(current_text_content_idx) = state.current_text_content_index() {
                    
                    // Look for run_event items at the same content position as current text
                    if let Some(_content_value) = state.node_data().content.get(current_text_content_idx) {
                        // Check if there's a run_event right before this text
                        if current_text_content_idx > 0 {
                            if let Some(prev_content) = state.node_data().content.get(current_text_content_idx - 1) {
                                if let Some(type_str) = prev_content.get("type").and_then(|v| v.as_str()) {
                                    if type_str == "run_event" {
                                        if let Some(index_override) = prev_content.get("index_override")
                                            .and_then(|v| serde_json::from_value::<mortar_compiler::IndexOverride>(v.clone()).ok()) {
                                            
                                            if let Some(event_name) = prev_content.get("name").and_then(|v| v.as_str()) {
                                                // Get the index value
                                                let index = if index_override.override_type == "variable" {
                                                    // Get variable value
                                                    variable_state
                                                        .get(&index_override.value)
                                                        .and_then(|v| {
                                                            if let bevy_mortar_bond::MortarVariableValue::Number(n) = v {
                                                                Some(*n)
                                                            } else {
                                                                None
                                                            }
                                                        })
                                                        .unwrap_or(0.0)
                                                } else {
                                                    // Direct value
                                                    index_override.value.parse::<f64>().unwrap_or(0.0)
                                                };

                                                // Find the event definition
                                                if let Some(event_def) = asset_data.events.iter().find(|e| e.name == event_name) {
                                                    // Create a text event from the event definition
                                                    let text_event = mortar_compiler::Event {
                                                        index,
                                                        index_variable: None,
                                                        actions: vec![event_def.action.clone()],
                                                    };
                                                    all_events.push(text_event);
                                                    info!(
                                                        "Example: Added run '{}' with index {} to text events",
                                                        event_name, index
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Add dialogue events tracking using library component
                if !all_events.is_empty() {
                    commands
                        .entity(entity)
                        .insert(MortarEventTracker::new(all_events));
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
    match action.action_name.as_str() {
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
        "play_sound" => {
            if let Some(file_name) = action.args.first() {
                commands.spawn(PendingAudioPlay(file_name.clone()));
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
    triangle_query: Query<(Entity, &Transform), With<TriangleSprite>>,
) {
    for (entity, pending) in &query {
        for (triangle_entity, transform) in triangle_query.iter() {
            match pending.0.as_str() {
                "wave" => {
                    commands.entity(triangle_entity).insert(RotateAnimation {
                        timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
                        start_rotation: 0.0,
                    });
                }
                "left" => {
                    let mut new_transform = *transform;
                    new_transform.translation.x = -50.0;
                    commands.entity(triangle_entity).insert(new_transform);
                }
                "right" => {
                    let mut new_transform = *transform;
                    new_transform.translation.x = 50.0;
                    commands.entity(triangle_entity).insert(new_transform);
                }
                _ => {}
            }
        }
        // Remove the marker component after processing
        //
        // 移除处理后的标记组件
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
                    info!("Example: Triangle color changed to {}", pending.0);
                }
            }
        }
        // Remove the marker component after processing
        //
        // 移除处理后的标记组件
        commands.entity(entity).remove::<PendingColorChange>();
    }
}

/// Play pending audio files
///
/// 播放待处理的音频文件
fn play_pending_audio(
    mut commands: Commands,
    query: Query<(Entity, &PendingAudioPlay)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, pending) in &query {
        info!("Example: Loading and playing audio: {}", pending.0);
        let audio_source = asset_server.load::<AudioSource>(pending.0.clone());
        commands.spawn(AudioPlayer::new(audio_source));

        // Remove the marker component after processing
        //
        // 移除处理后的标记组件
        commands.entity(entity).despawn();
    }
}

/// Component for rotation animation
///
/// 旋转动画的组件
#[derive(Component)]
struct RotateAnimation {
    timer: Timer,
    start_rotation: f32,
}

/// System to update rotation animations
///
/// 更新旋转动画的系统
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

/// Process pending run executions with duration
///
/// 处理带有持续时间的待执行run
fn process_pending_run_executions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PendingRunExecution)>,
    mut runtime: ResMut<MortarRuntime>,
) {
    for (entity, mut pending) in &mut query {
        pending.timer.tick(time.delta());

        if pending.timer.is_finished() {
            // Execute next event in sequence
            if let Some((event_name, _, _)) = pending.remaining_runs.first() {
                if event_name != "__WAIT__" {
                    if let Some(event_def) =
                        pending.event_defs.iter().find(|e| e.name == *event_name)
                    {
                        execute_event_action(
                            &event_def.action,
                            &mut commands,
                            pending.dialogue_entity,
                        );
                    }
                }
            }

            // Remove first event and continue
            if pending.remaining_runs.len() > 1 {
                let remaining = pending.remaining_runs[1..].to_vec();
                let next_event = &pending.remaining_runs[0];

                // Calculate next duration
                let duration_secs = if next_event.0 == "__WAIT__" {
                    next_event.1.unwrap_or(0.0)
                } else if next_event.2 {
                    // ignore_duration is true ("now run")
                    0.0
                } else {
                    pending
                        .event_defs
                        .iter()
                        .find(|e| e.name == next_event.0)
                        .and_then(|e| e.duration)
                        .unwrap_or(0.0)
                };

                if duration_secs > 0.0 {
                    // Update timer for next event
                    pending.remaining_runs = remaining;
                    pending.timer = Timer::from_seconds(duration_secs as f32, TimerMode::Once);
                } else {
                    // No duration, spawn new execution for remaining
                    let event_defs = pending.event_defs.clone();
                    let timeline_defs = pending.timeline_defs.clone();
                    let dialogue_entity = pending.dialogue_entity;
                    commands.entity(entity).despawn();
                    start_timeline_execution(
                        remaining,
                        event_defs,
                        timeline_defs,
                        dialogue_entity,
                        &mut commands,
                    );
                }
            } else {
                // Timeline complete, trigger runtime change to show next text
                commands.entity(entity).despawn();
                info!("Example: Run sequence completed, ready to show next text");
                // Touch the runtime to mark it as changed
                if let Some(_state) = &runtime.active_dialogue {
                    runtime.set_changed();
                }
            }
        }
    }
}

/// System to clear runs executing flag when all pending runs are done
///
/// 当所有待执行runs完成时清除执行标志的系统
fn clear_runs_executing_flag(
    pending_runs_query: Query<&PendingRunExecution>,
    mut runs_executing: ResMut<RunsExecuting>,
    mut runtime: ResMut<MortarRuntime>,
) {
    if runs_executing.executing && pending_runs_query.is_empty() {
        info!("Example: All runs completed, clearing flag");
        runs_executing.executing = false;
        // Touch runtime to trigger text update
        if runtime.active_dialogue.is_some() {
            runtime.set_changed();
        }
    }
}

/// Parse hex color string like "#FF6B6B"
///
/// 转换十六进制颜色字符串，如 "#FF6B6B"
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

/// Execute a run statement by name - either an event or a timeline
///
/// 通过名称执行run语句 - 事件或时间轴
fn execute_run_by_name(
    event_name: &str,
    event_defs: &[mortar_compiler::EventDef],
    timeline_defs: &[mortar_compiler::TimelineDef],
    commands: &mut Commands,
    entity: Entity,
) {
    // First try to find as an event
    if let Some(event_def) = event_defs.iter().find(|e| e.name == event_name) {
        info!("Example: Executing event: {}", event_name);
        execute_event_action(&event_def.action, commands, entity);
        return;
    }

    // Then try to find as a timeline
    if let Some(timeline_def) = timeline_defs.iter().find(|t| t.name == event_name) {
        info!("Example: Executing timeline: {}", event_name);

        // Build a list of (event_name, duration, ignore_duration) tuples from the timeline
        let mut timeline_sequence = Vec::new();
        for stmt in &timeline_def.statements {
            match stmt.stmt_type.as_str() {
                "run" => {
                    if let Some(event_name) = &stmt.event_name {
                        // Check if ignore_duration is set (now run)
                        let duration = if stmt.ignore_duration {
                            None // Ignore duration completely
                        } else {
                            stmt.duration
                        };
                        timeline_sequence.push((
                            event_name.clone(),
                            duration,
                            stmt.ignore_duration,
                        ));
                    }
                }
                "wait" => {
                    // Wait statements add delay to the next event
                    if let Some(duration) = stmt.duration {
                        timeline_sequence.push(("__WAIT__".to_string(), Some(duration), false));
                    }
                }
                _ => {}
            }
        }

        // Start executing the timeline sequence
        if !timeline_sequence.is_empty() {
            start_timeline_execution(
                timeline_sequence,
                event_defs.to_vec(),
                timeline_defs.to_vec(),
                entity,
                commands,
            );
        }
        return;
    }

    warn!("Example: Run statement target not found: {}", event_name);
}

/// Start executing a timeline sequence with durations
///
/// 开始执行带有持续时间的时间轴序列
fn start_timeline_execution(
    sequence: Vec<(String, Option<f64>, bool)>,
    event_defs: Vec<mortar_compiler::EventDef>,
    timeline_defs: Vec<mortar_compiler::TimelineDef>,
    dialogue_entity: Entity,
    commands: &mut Commands,
) {
    // Execute the first event immediately
    if let Some((first_event, first_duration, ignore_duration)) = sequence.first() {
        if first_event != "__WAIT__" {
            if let Some(event_def) = event_defs.iter().find(|e| e.name == *first_event) {
                execute_event_action(&event_def.action, commands, dialogue_entity);
            }
        }

        // If there are more events and the first has a duration, schedule them
        if sequence.len() > 1 {
            let remaining = sequence[1..].to_vec();
            let duration_secs = if first_event == "__WAIT__" {
                first_duration.unwrap_or(0.0)
            } else if *ignore_duration {
                // "now run" - ignore duration completely
                0.0
            } else {
                // Use event's duration or 0
                event_defs
                    .iter()
                    .find(|e| e.name == *first_event)
                    .and_then(|e| e.duration)
                    .unwrap_or(0.0)
            };

            if duration_secs > 0.0 {
                commands.spawn(PendingRunExecution {
                    timer: Timer::from_seconds(duration_secs as f32, TimerMode::Once),
                    remaining_runs: remaining,
                    event_defs,
                    timeline_defs,
                    dialogue_entity,
                });
            } else {
                // No duration, continue immediately
                start_timeline_execution(
                    remaining,
                    event_defs,
                    timeline_defs,
                    dialogue_entity,
                    commands,
                );
            }
        }
    }
}

/// Execute an event action
///
/// 执行事件动作
fn execute_event_action(action: &mortar_compiler::Action, commands: &mut Commands, entity: Entity) {
    // Parse args - remove quotes if present
    let parsed_args: Vec<String> = action
        .args
        .iter()
        .map(|arg| arg.trim_matches('"').to_string())
        .collect();

    let mortar_action = MortarEventAction {
        action_name: action.action_type.clone(),
        args: parsed_args,
    };

    handle_mortar_action(entity, mortar_action, commands);
}
