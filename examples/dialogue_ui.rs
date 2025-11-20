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
                update_dialogue_text,
                update_button_states,
            ),
        )
        .run();
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ChoiceButton {
    index: usize,
}

#[derive(Component)]
struct ContinueButton;

fn load_initial_dialogue(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
) {
    let path = "Demo.mortar".to_string();
    info!("开始加载文件: {}", &path);
    let handle = asset_server.load(&path);
    registry.register(path.clone(), handle);

    info!("发送 StartNode 事件: {} / Start", &path);
    events.write(MortarEvent::StartNode {
        path,
        node: "Start".to_string(),
    });
}

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

fn button_interaction_system(
    mut continue_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
) {
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
}

fn handle_continue_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut events: MessageWriter<MortarEvent>,
    runtime: Res<MortarRuntime>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(state) = &runtime.active_dialogue {
                if state.has_next_text() {
                    events.write(MortarEvent::NextText);
                } else {
                    info!("节点 '{}' 已结束", state.current_node);
                }
            }
        }
    }
}

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

fn update_button_states(
    runtime: Res<MortarRuntime>,
    mut button_query: Query<&mut Text, With<ContinueButton>>,
) {
    if !runtime.is_changed() {
        return;
    }

    for mut text in button_query.iter_mut() {
        if let Some(state) = &runtime.active_dialogue {
            **text = if state.has_next_text() {
                "继续".to_string()
            } else {
                "已结束".to_string()
            };
        } else {
            **text = "继续".to_string();
        }
    }
}
