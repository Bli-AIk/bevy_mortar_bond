use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (button_interaction_system, handle_choice_button))
        .run();
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ChoiceButton {
    index: usize,
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

            let font_clone = font.clone();
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
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
                                BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                                ChoiceButton { index: i },
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("选项 {}", i + 1)),
                                    TextFont {
                                        font: font_clone.clone(),
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                ));
                            });
                    }
                });
        });
}

fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ChoiceButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.55, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.9, 0.7));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.45));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.7, 0.9));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.5));
            }
        }
    }
}

fn handle_choice_button(
    interaction_query: Query<(&Interaction, &ChoiceButton), Changed<Interaction>>,
    mut dialogue_query: Query<&mut Text, With<DialogueText>>,
) {
    for (interaction, choice) in &interaction_query {
        if *interaction == Interaction::Pressed {
            for mut text in &mut dialogue_query {
                **text = format!("你点击了选项 {}！\n这个选择将会触发相应的对话节点。", choice.index + 1);
            }
        }
    }
}
