use bevy::prelude::*;
use bevy_mortar_bond::{MortarPlugin, MortarRegistry, MortarRuntime, MortarEvent};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MortarPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            button_interaction_system,
            handle_continue_button,
            handle_load_button,
            update_dialogue_text,
            update_button_states,
            handle_text_input,
        ))
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

#[derive(Component)]
struct LoadButton;

#[derive(Component)]
struct FileInput;

#[derive(Resource, Default)]
struct InputState {
    file_path: String,
    is_focused: bool,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    
    // 初始化输入状态，默认文件名为 Demo.mortar
    commands.insert_resource(InputState {
        file_path: "Demo.mortar".to_string(),
        is_focused: false,
    });

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
            // 文件加载区域
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // 文件输入框
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(70.0),
                                height: Val::Px(50.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                            Interaction::None,
                            FileInput,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Demo.mortar"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // 加载按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(30.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.4, 0.5)),
                            BorderColor::all(Color::srgb(0.5, 0.6, 0.7)),
                            LoadButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("加载文件"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });
                });

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
                        Text::new("欢迎来到 Mortar 对话系统演示！\n这是一个简单的对话显示区域。"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        DialogueText,
                    ));
                });

            // 选项按钮（默认禁用）
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
    mut queries: ParamSet<(
        Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor),
            (Changed<Interaction>, With<ContinueButton>),
        >,
        Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor),
            (Changed<Interaction>, With<LoadButton>),
        >,
        Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor),
            (Changed<Interaction>, With<FileInput>),
        >,
    )>,
    mut input_state: ResMut<InputState>,
) {
    // 继续按钮交互
    for (interaction, mut bg_color, mut border_color) in queries.p0().iter_mut() {
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

    // 加载按钮交互
    for (interaction, mut bg_color, mut border_color) in queries.p1().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.3, 0.4));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.5, 0.6));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.45, 0.55));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.6, 0.7));
            }
        }
    }

    // 输入框交互
    for (interaction, mut bg_color, mut border_color) in queries.p2().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                input_state.is_focused = true;
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.6, 0.8));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.6, 0.6));
            }
            Interaction::None => {
                if !input_state.is_focused {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                    *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.5));
                }
            }
        }
    }
}

fn handle_continue_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut events: MessageWriter<MortarEvent>,
    runtime: Res<MortarRuntime>,
    mut input_state: ResMut<InputState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            input_state.is_focused = false;
            
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

fn handle_load_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    mut input_state: ResMut<InputState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            input_state.is_focused = false;
            
            if input_state.file_path.is_empty() {
                warn!("文件路径为空");
                continue;
            }

            let path = input_state.file_path.clone();
            
            info!("尝试加载文件: {}", path);
            
            // 检查是否已注册
            if registry.get(&path).is_none() {
                info!("文件未注册，开始加载: {}", path);
                let handle = asset_server.load(&path);
                registry.register(path.clone(), handle);
            } else {
                info!("文件已注册: {}", path);
            }

            // 启动该文件的 Start 节点
            info!("发送 StartNode 事件: {} / Start", path);
            events.write(MortarEvent::StartNode {
                path,
                node: "Start".to_string(),
            });
        }
    }
}

fn handle_text_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<InputState>,
    mut input_query: Query<&mut Text, With<FileInput>>,
) {
    if !input_state.is_focused {
        return;
    }

    let mut modified = false;

    // 处理退格键
    if keys.just_pressed(KeyCode::Backspace) {
        input_state.file_path.pop();
        modified = true;
    }

    // 处理回车键（失去焦点）
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Escape) {
        input_state.is_focused = false;
        modified = true;
    }

    // 处理常见字符输入
    for key in keys.get_just_pressed() {
        let c = match key {
            KeyCode::KeyA => Some('a'),
            KeyCode::KeyB => Some('b'),
            KeyCode::KeyC => Some('c'),
            KeyCode::KeyD => Some('d'),
            KeyCode::KeyE => Some('e'),
            KeyCode::KeyF => Some('f'),
            KeyCode::KeyG => Some('g'),
            KeyCode::KeyH => Some('h'),
            KeyCode::KeyI => Some('i'),
            KeyCode::KeyJ => Some('j'),
            KeyCode::KeyK => Some('k'),
            KeyCode::KeyL => Some('l'),
            KeyCode::KeyM => Some('m'),
            KeyCode::KeyN => Some('n'),
            KeyCode::KeyO => Some('o'),
            KeyCode::KeyP => Some('p'),
            KeyCode::KeyQ => Some('q'),
            KeyCode::KeyR => Some('r'),
            KeyCode::KeyS => Some('s'),
            KeyCode::KeyT => Some('t'),
            KeyCode::KeyU => Some('u'),
            KeyCode::KeyV => Some('v'),
            KeyCode::KeyW => Some('w'),
            KeyCode::KeyX => Some('x'),
            KeyCode::KeyY => Some('y'),
            KeyCode::KeyZ => Some('z'),
            KeyCode::Digit0 => Some('0'),
            KeyCode::Digit1 => Some('1'),
            KeyCode::Digit2 => Some('2'),
            KeyCode::Digit3 => Some('3'),
            KeyCode::Digit4 => Some('4'),
            KeyCode::Digit5 => Some('5'),
            KeyCode::Digit6 => Some('6'),
            KeyCode::Digit7 => Some('7'),
            KeyCode::Digit8 => Some('8'),
            KeyCode::Digit9 => Some('9'),
            KeyCode::Period => Some('.'),
            KeyCode::Slash => Some('/'),
            KeyCode::Minus => Some('-'),
            KeyCode::Space => Some(' '),
            _ => None,
        };

        if let Some(ch) = c {
            input_state.file_path.push(ch);
            modified = true;
        }
    }

    // 更新输入框显示
    if modified || input_state.is_changed() {
        for mut text in input_query.iter_mut() {
            **text = if input_state.file_path.is_empty() {
                if input_state.is_focused {
                    "_".to_string()
                } else {
                    "Demo.mortar".to_string()
                }
            } else {
                if input_state.is_focused {
                    format!("{}|", input_state.file_path)
                } else {
                    input_state.file_path.clone()
                }
            };
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
                    state.mortar_path,
                    state.current_node,
                    current_text
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
                "下一个节点".to_string()
            };
        }
    }
}
