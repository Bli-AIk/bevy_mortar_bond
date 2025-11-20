//! A simple dialogue UI example for the `bevy_mortar_bond` crate.
//!
//! `bevy_mortar_bond` 包的一个简单对话 UI 示例。

use bevy::prelude::*;
use bevy_mortar_bond::{MortarEvent, MortarPlugin, MortarRegistry, MortarRuntime};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MortarPlugin))
        .add_systems(Startup, (setup, load_initial_dialogue).chain())
        .add_systems(
            Update,
            (
                button_interaction_system,
                handle_continue_button,
                handle_choice_buttons,
                update_dialogue_text,
                update_button_states,
            ),
        )
        .run();
}

/// A component for the dialogue text UI element.
///
/// 对话文本 UI 元素的组件。
#[derive(Component)]
struct DialogueText;

/// A component for choice buttons in the UI.
///
/// UI 中选项按钮的组件。
#[derive(Component)]
struct ChoiceButton {
    index: usize,
}

/// A component for the "Continue" button.
///
/// “继续”按钮的组件。
#[derive(Component)]
struct ContinueButton;

/// Loads the initial dialogue file and starts the first node.
///
/// 加载初始对话文件并启动第一个节点。
fn load_initial_dialogue(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
) {
    let path = "Demo.mortar".to_string();
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

/// Sets up the UI for the dialogue.
///
/// 设置对话的 UI。
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let font = asset_server.load("Unifont.otf");

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 对话文本区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(150.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        margin: UiRect::bottom(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("欢迎来到 Mortar 对话系统演示！\n正在加载 'Demo.mortar'..."),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        DialogueText,
                    ));
                });

            // 选项按钮
            let font_clone = font.clone();
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .with_children(move |parent| {
                    for i in 0..3 {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                                ChoiceButton { index: i },
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("选项 {} (禁用)", i + 1)),
                                    TextFont {
                                        font: font_clone.clone(),
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                                ));
                            });
                    }
                });

            // 继续按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                    BorderColor::all(Color::srgb(0.5, 0.7, 0.5)),
                    ContinueButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("继续"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

/// Handles the visual feedback for button interactions.
///
/// 处理按钮交互的视觉反馈。
fn button_interaction_system(
    mut continue_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<ContinueButton>,
            Without<ChoiceButton>,
        ),
    >,
    mut choice_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ChoiceButton>),
    >,
) {
    // Continue button interaction
    for (interaction, mut bg_color, mut border_color) in continue_button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.45, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.8, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.55, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.8, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.5, 0.3));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.7, 0.5));
            }
        }
    }

    // Choice button interaction
    for (interaction, mut bg_color, mut border_color) in choice_button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.5));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.6, 0.8));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.45, 0.6));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.7, 0.9));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.4, 0.55));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.6, 0.75));
            }
        }
    }
}

/// Handles clicks on the "Continue" button.
///
/// 处理“继续”按钮的点击事件。
fn handle_continue_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut events: MessageWriter<MortarEvent>,
    runtime: Res<MortarRuntime>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed
            && let Some(state) = &runtime.active_dialogue
        {
            // Always send NextText event, system will handle node transitions
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

/// Updates the dialogue text display.
///
/// 更新对话文本显示。
fn update_dialogue_text(
    runtime: Res<MortarRuntime>,
    mut dialogue_query: Query<&mut Text, With<DialogueText>>,
) {
    if !runtime.is_changed() {
        return;
    }

    for mut text in &mut dialogue_query {
        if let Some(state) = &runtime.active_dialogue {
            if let Some(current_text) = state.current_text() {
                **text = format!(
                    "[{} / {}]\n\n{}",
                    state.mortar_path, state.current_node, current_text
                );
            }
        } else {
            **text = "等待加载对话...".to_string();
        }
    }
}

/// Updates the state of the buttons based on the dialogue state.
///
/// 根据对话状态更新按钮的状态。
fn update_button_states(
    runtime: Res<MortarRuntime>,
    mut continue_query: Query<
        (&mut Text, &mut Visibility),
        (With<ContinueButton>, Without<ChoiceButton>),
    >,
    mut choice_query: Query<
        (&ChoiceButton, &mut Text, &mut Visibility, &Children),
        Without<ContinueButton>,
    >,
    mut text_query: Query<&mut TextColor>,
) {
    if !runtime.is_changed() {
        return;
    }

    // Update continue button
    for (mut text, mut visibility) in continue_query.iter_mut() {
        if let Some(state) = &runtime.active_dialogue {
            if state.has_choices() && !state.has_next_text() {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Visible;
                **text = if state.has_next_text() {
                    "继续".to_string()
                } else {
                    "继续".to_string()
                };
            }
        } else {
            *visibility = Visibility::Visible;
            **text = "继续".to_string();
        }
    }

    // Update choice buttons
    if let Some(state) = &runtime.active_dialogue {
        if let Some(choices) = state.get_choices()
            && !state.has_next_text()
        {
            for (choice_button, mut text, mut visibility, children) in choice_query.iter_mut() {
                if let Some(choice) = choices.get(choice_button.index) {
                    *visibility = Visibility::Visible;
                    **text = choice.text.clone();

                    // Update text color to active
                    for child in children.iter() {
                        if let Ok(mut text_color) = text_query.get_mut(child) {
                            *text_color = TextColor(Color::srgb(0.9, 0.9, 0.9));
                        }
                    }
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        } else {
            // Hide all choice buttons when not needed
            for (_, _, mut visibility, children) in choice_query.iter_mut() {
                *visibility = Visibility::Hidden;

                // Reset text color
                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        *text_color = TextColor(Color::srgb(0.4, 0.4, 0.4));
                    }
                }
            }
        }
    } else {
        // Hide all choice buttons when no active dialogue
        for (_, _, mut visibility, _) in choice_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
