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
            handle_file_select_button,
            update_dialogue_text,
            update_button_states,
            update_file_display,
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
struct FileSelectButton;

#[derive(Resource)]
struct FileListState {
    files: Vec<String>,
    current_index: usize,
}

impl Default for FileListState {
    fn default() -> Self {
        // 扫描 assets 目录下的 .mortar 文件
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir("assets") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".mortar") {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        files.sort();
        Self {
            files,
            current_index: 0,
        }
    }
}

impl FileListState {
    fn current_file(&self) -> Option<&str> {
        self.files.get(self.current_index).map(|s| s.as_str())
    }
    
    fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.current_index = (self.current_index + 1) % self.files.len();
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    
    // 初始化文件列表
    commands.init_resource::<FileListState>();

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
                    // 文件选择按钮（点击切换）
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(70.0),
                                height: Val::Px(50.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
                            BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                            FileSelectButton,
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
            (Changed<Interaction>, With<FileSelectButton>),
        >,
    )>,
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

    // 文件选择按钮交互
    for (interaction, mut bg_color, mut border_color) in queries.p2().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                *border_color = BorderColor::all(Color::srgb(0.6, 0.6, 0.7));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
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

fn handle_file_select_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<FileSelectButton>)>,
    mut file_state: ResMut<FileListState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            file_state.next_file();
            info!("切换到文件: {:?}", file_state.current_file());
        }
    }
}

fn handle_load_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut runtime: ResMut<MortarRuntime>,
    mut events: MessageWriter<MortarEvent>,
    file_state: Res<FileListState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let Some(file_name) = file_state.current_file() else {
                warn!("没有可用的文件");
                continue;
            };

            let path = file_name.to_string();
            
            info!("尝试加载文件: {}", path);
            
            // 重置当前对话状态
            runtime.active_dialogue = None;
            runtime.pending_start = None;
            
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

// 更新文件选择按钮的显示
fn update_file_display(
    file_state: Res<FileListState>,
    mut button_query: Query<&mut Text, With<FileSelectButton>>,
) {
    if !file_state.is_changed() {
        return;
    }
    
    for mut text in button_query.iter_mut() {
        **text = if let Some(file) = file_state.current_file() {
            file.to_string()
        } else {
            "无可用文件".to_string()
        };
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
                "已结束".to_string()
            };
        } else {
            **text = "继续".to_string();
        }
    }
}
