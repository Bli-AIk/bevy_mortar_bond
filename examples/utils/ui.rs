//! Common UI components for dialogue examples.
//!
//! This module provides pure UI components without dependencies on bevy_mortar_bond.
//!
//! 对话示例的通用 UI 组件。
//!
//! 此模块提供纯 UI 组件，不依赖 bevy_mortar_bond。

use bevy::prelude::*;

/// A component for the dialogue text UI element.
///
/// 对话文本 UI 元素的组件。
#[derive(Component)]
pub struct DialogueText;

/// A component for choice buttons in the UI.
///
/// UI 中选项按钮的组件。
#[derive(Component)]
pub struct ChoiceButton {
    pub index: usize,
}

/// A marker component for the choice buttons container.
///
/// 选项按钮容器的标记组件。
#[derive(Component)]
pub struct ChoiceContainer;

/// A component for the "Continue" button.
///
/// "继续"按钮的组件。
#[derive(Component)]
pub struct ContinueButton;

/// A component for the "Reload" button.
///
/// "重载"按钮的组件。
#[derive(Component)]
pub struct ReloadButton;

/// A component for the "Switch File" button.
///
/// "切换文件"按钮的组件。
#[derive(Component)]
pub struct SwitchFileButton;

/// Creates the dialogue UI layout.
///
/// 创建对话 UI 布局。
pub fn setup_dialogue_ui(commands: &mut Commands, font: Handle<Font>) {
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
            // Dialogue text area
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

            // Choice buttons container (initially empty, will be populated dynamically)
            parent.spawn((
                Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                ChoiceContainer,
            ));

            // Continue button
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

            // Control buttons row
            let font_clone = font.clone();
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                })
                .with_children(move |parent| {
                    // Reload button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.4, 0.2)),
                            BorderColor::all(Color::srgb(0.7, 0.6, 0.4)),
                            ReloadButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("重载当前文件"),
                                TextFont {
                                    font: font_clone.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // Switch file button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.3, 0.5)),
                            BorderColor::all(Color::srgb(0.6, 0.5, 0.7)),
                            SwitchFileButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("切换文件"),
                                TextFont {
                                    font: font_clone.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });
                });
        });
}

/// Handles the visual feedback for button interactions.
///
/// 处理按钮交互的视觉反馈。
pub fn button_interaction_system(
    mut continue_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<ContinueButton>,
            Without<ChoiceButton>,
            Without<ReloadButton>,
            Without<SwitchFileButton>,
        ),
    >,
    mut choice_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<ChoiceButton>,
            Without<ContinueButton>,
            Without<ReloadButton>,
            Without<SwitchFileButton>,
        ),
    >,
    mut reload_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<ReloadButton>,
            Without<ContinueButton>,
            Without<ChoiceButton>,
            Without<SwitchFileButton>,
        ),
    >,
    mut switch_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<SwitchFileButton>,
            Without<ContinueButton>,
            Without<ChoiceButton>,
            Without<ReloadButton>,
        ),
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

    // Choice button interaction - skip if already set by manage_choice_buttons
    // (don't override selected/disabled states)
    for (interaction, _bg_color, _border_color) in choice_button_query.iter_mut() {
        // Only respond to hover/press on normal (non-selected) buttons
        // The actual color changes are handled by manage_choice_buttons
        match *interaction {
            Interaction::Pressed => {}
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }

    // Reload button interaction
    for (interaction, mut bg_color, mut border_color) in reload_button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.45, 0.35, 0.15));
                *border_color = BorderColor::all(Color::srgb(0.8, 0.7, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.55, 0.45, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.8, 0.7, 0.5));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.4, 0.2));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.6, 0.4));
            }
        }
    }

    // Switch file button interaction
    for (interaction, mut bg_color, mut border_color) in switch_button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.25, 0.45));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.6, 0.8));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.45, 0.35, 0.55));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.6, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.3, 0.5));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.5, 0.7));
            }
        }
    }
}
