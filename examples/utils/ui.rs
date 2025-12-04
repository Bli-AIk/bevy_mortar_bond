//! UI helpers and plugin for the dialogue example.
//!
//! All layout + button handling is kept here so the example file can focus on binding logic.
//!
//! 对话示例的 UI 插件与组件。
//!
//! 这里集中处理布局和按钮交互，示例文件可专注于讲解绑定。

use super::typewriter::{Typewriter, TypewriterState};
use bevy::asset::Assets;
use bevy::ecs::system::{Local, SystemParam};
use bevy::log::info;
use bevy::prelude::*;
use bevy::ui::FlexDirection;
use bevy_mortar_bond::{
    DialogueState, MortarAsset, MortarDialogueSystemSet, MortarDialogueText, MortarEvent,
    MortarEventBinding, MortarRegistry, MortarRunsExecuting, MortarRuntime, MortarTextTarget,
};

use crate::DialogueFiles;

const TYPEWRITER_SPEED: f32 = 0.04;
const FINISHED_TEXT: &str = "该对话已结束";

/// UI plugin bundling layout + button logic for dialogue examples.
///
/// 用于对话示例的 UI 插件，封装布局及按钮逻辑。
pub struct DialogueUiPlugin;

impl Plugin for DialogueUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_dialogue_ui)
            .add_systems(
                Update,
                (
                    button_interaction_system,
                    handle_continue_button,
                    handle_choice_buttons,
                    handle_reload_button,
                    handle_switch_file_button,
                    manage_choice_buttons,
                    update_choice_button_styles,
                    update_button_states,
                ),
            )
            .add_systems(
                Update,
                (
                    sync_typewriter_with_dialogue_texts.after(MortarDialogueSystemSet::UpdateText),
                    update_event_binding_from_typewriter
                        .after(sync_typewriter_with_dialogue_texts)
                        .before(MortarDialogueSystemSet::TriggerEvents),
                ),
            )
            .add_systems(PostUpdate, apply_typewriter_output_to_texts);
    }
}

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

type InteractionColor<'a> = (
    &'a Interaction,
    &'a mut BackgroundColor,
    &'a mut BorderColor,
);

type ContinueButtonFilter = (
    Changed<Interaction>,
    With<ContinueButton>,
    Without<ChoiceButton>,
    Without<ReloadButton>,
    Without<SwitchFileButton>,
);

type ChoiceButtonFilter = (
    Changed<Interaction>,
    With<ChoiceButton>,
    Without<ContinueButton>,
    Without<ReloadButton>,
    Without<SwitchFileButton>,
);

type ReloadButtonFilter = (
    Changed<Interaction>,
    With<ReloadButton>,
    Without<ContinueButton>,
    Without<ChoiceButton>,
    Without<SwitchFileButton>,
);

type SwitchButtonFilter = (
    Changed<Interaction>,
    With<SwitchFileButton>,
    Without<ContinueButton>,
    Without<ChoiceButton>,
    Without<ReloadButton>,
);

#[derive(SystemParam)]
struct ChoiceButtonResources<'w> {
    asset_server: Res<'w, AssetServer>,
    registry: Res<'w, MortarRegistry>,
    assets: Res<'w, Assets<MortarAsset>>,
}

/// Snapshot of choice selection state to detect changes.
///
/// 记录选项状态快照以便检测变化。
#[derive(Clone, Default, PartialEq)]
struct ChoiceUiSnapshot {
    mortar_path: String,
    node_name: String,
    choice_stack: Vec<usize>,
    choices_broken: bool,
}

type ChoiceUiState = Option<ChoiceUiSnapshot>;

/// Creates the dialogue UI layout.
///
/// 创建对话 UI 布局。
pub fn setup_dialogue_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("font/Unifont.otf");

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
            // Dialogue text area.
            //
            // 对话文本区域。
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
                        MortarTextTarget,
                        Typewriter::new("", TYPEWRITER_SPEED),
                    ));
                });

            // Choice buttons container (initially empty, will be populated dynamically).
            //
            // 选项按钮容器（初始为空，将在运行时填充）。
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

            // Continue button.
            //
            // “继续”按钮。
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

            // Control buttons row.
            //
            // 控制按钮行。
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
                    // Reload button.
                    //
                    // “重载”按钮。
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

                    // Switch file button.
                    //
                    // “切换文件”按钮。
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

fn sync_typewriter_with_dialogue_texts(
    mut query: Query<
        (&MortarDialogueText, &mut Typewriter),
        (Changed<MortarDialogueText>, With<DialogueText>),
    >,
) {
    for (dialogue_text, mut typewriter) in &mut query {
        *typewriter = Typewriter::new(dialogue_text.body.clone(), TYPEWRITER_SPEED);
        typewriter.play();
    }
}

fn apply_typewriter_output_to_texts(
    mut query: Query<(&Typewriter, &MortarDialogueText, &mut Text), With<DialogueText>>,
) {
    for (typewriter, dialogue_text, mut text) in &mut query {
        **text = format!("{}{}", dialogue_text.header, typewriter.current_text);
    }
}

fn update_event_binding_from_typewriter(
    mut query: Query<(&Typewriter, Option<&mut MortarEventBinding>), With<DialogueText>>,
) {
    for (typewriter, binding) in &mut query {
        if let Some(mut binding) = binding {
            binding.current_index = typewriter.current_char_index as f32;
        }
    }
}

fn show_finished_message(
    dialogue_text_query: &mut Query<&mut MortarDialogueText, With<DialogueText>>,
    text_query: &mut Query<&mut Text, With<DialogueText>>,
    typewriter_query: &mut Query<&mut Typewriter, With<DialogueText>>,
) {
    if let Ok(mut dialogue_text) = dialogue_text_query.single_mut() {
        dialogue_text.header.clear();
        dialogue_text.body = FINISHED_TEXT.to_string();
    }

    if let Ok(mut text) = text_query.single_mut() {
        **text = FINISHED_TEXT.to_string();
    }

    if let Ok(mut typewriter) = typewriter_query.single_mut() {
        typewriter.source_text = FINISHED_TEXT.to_string();
        typewriter.current_text = FINISHED_TEXT.to_string();
        typewriter.current_char_index = FINISHED_TEXT.chars().count();
        typewriter.timer.reset();
        typewriter.state = TypewriterState::Finished;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ChoiceFinishKind {
    None,
    Immediate,
    NeedsNextText,
}

fn determine_choice_finish_kind(state: &DialogueState, choice_index: usize) -> ChoiceFinishKind {
    let Some(choices) = state.get_current_choices().or_else(|| state.get_choices()) else {
        return ChoiceFinishKind::None;
    };
    let Some(choice) = choices.get(choice_index) else {
        return ChoiceFinishKind::None;
    };

    if choice.choice.is_some() {
        return ChoiceFinishKind::None;
    }

    if let Some(action) = choice.action.as_deref() {
        return match action {
            "return" => ChoiceFinishKind::Immediate,
            "break" => {
                let has_more_text = state.has_next_text();
                let has_follow_up_node =
                    matches!(state.get_next_node(), Some(next) if next != "return");
                if !has_more_text && !has_follow_up_node {
                    ChoiceFinishKind::NeedsNextText
                } else {
                    ChoiceFinishKind::None
                }
            }
            _ => ChoiceFinishKind::Immediate,
        };
    }

    if let Some(next_node) = choice.next.as_deref() {
        if next_node == "return" {
            ChoiceFinishKind::Immediate
        } else {
            ChoiceFinishKind::None
        }
    } else {
        ChoiceFinishKind::Immediate
    }
}

/// Handles the visual feedback for button interactions.
///
/// 处理按钮交互的视觉反馈。
pub fn button_interaction_system(
    mut continue_button_query: Query<InteractionColor<'_>, ContinueButtonFilter>,
    mut choice_button_query: Query<InteractionColor<'_>, ChoiceButtonFilter>,
    mut reload_button_query: Query<InteractionColor<'_>, ReloadButtonFilter>,
    mut switch_button_query: Query<InteractionColor<'_>, SwitchButtonFilter>,
) {
    // Continue button interaction.
    //
    // 处理“继续”按钮交互。
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

    // Choice button interaction - skip if already set by manage_choice_buttons.
    // (don't override selected/disabled states)
    //
    // 选项按钮交互——若 manage_choice_buttons 已设置则跳过。
    // （不要覆盖选中/禁用状态）
    for (interaction, _bg_color, _border_color) in choice_button_query.iter_mut() {
        // Only respond to hover/press on normal (non-selected) buttons.
        // The actual color changes are handled by manage_choice_buttons.
        //
        // 仅响应普通（未选中）按钮的悬停/按压。
        // 颜色变化由 manage_choice_buttons 负责。
        match *interaction {
            Interaction::Pressed => {}
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }

    // Reload button interaction.
    //
    // 处理“重载”按钮交互。
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

    // Switch file button interaction.
    //
    // 处理“切换文件”按钮交互。
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

/// Handles clicks on the "Continue" button.
///
/// 处理“继续”按钮点击。
fn handle_continue_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut events: MessageWriter<MortarEvent>,
    runtime: Res<MortarRuntime>,
    runs_executing: Res<MortarRunsExecuting>,
    mut dialogue_text_query: Query<&mut MortarDialogueText, With<DialogueText>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
    mut typewriter_query: Query<&mut Typewriter, With<DialogueText>>,
) {
    if runs_executing.executing {
        return;
    }

    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(state) = &runtime.active_dialogue {
            if let Some(choice_index) = state.selected_choice {
                info!("Example: Confirming choice selection");
                let finish_kind = determine_choice_finish_kind(state, choice_index);
                events.write(MortarEvent::ConfirmChoice);
                match finish_kind {
                    ChoiceFinishKind::Immediate => {
                        info!("Example: Choice ends dialogue immediately");
                        show_finished_message(
                            &mut dialogue_text_query,
                            &mut text_query,
                            &mut typewriter_query,
                        );
                    }
                    ChoiceFinishKind::NeedsNextText => {
                        info!("Example: Choice ends dialogue after break; advancing");
                        events.write(MortarEvent::NextText);
                        show_finished_message(
                            &mut dialogue_text_query,
                            &mut text_query,
                            &mut typewriter_query,
                        );
                    }
                    ChoiceFinishKind::None => {}
                }
                continue;
            }

            if state.has_next_text() {
                events.write(MortarEvent::NextText);
            } else if state.has_choices() && !state.choices_broken {
                info!("Example: Waiting for choice resolution before finishing");
            } else {
                events.write(MortarEvent::NextText);
                info!("Example: Dialogue finished; showing end message");
                show_finished_message(
                    &mut dialogue_text_query,
                    &mut text_query,
                    &mut typewriter_query,
                );
            }
        } else {
            show_finished_message(
                &mut dialogue_text_query,
                &mut text_query,
                &mut typewriter_query,
            );
        }
    }
}

/// Handles clicks on choice buttons.
///
/// 处理选项按钮点击。
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

/// Dynamically creates/updates choice buttons based on dialogue state.
///
/// 根据对话状态动态创建/更新选项按钮。
fn manage_choice_buttons(
    mut commands: Commands,
    runtime: Res<MortarRuntime>,
    container_query: Query<Entity, With<ChoiceContainer>>,
    button_query: Query<Entity, With<ChoiceButton>>,
    resources: ChoiceButtonResources,
    mut last_state: Local<ChoiceUiState>,
) {
    if !runtime.is_changed() {
        return;
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    let current_state = runtime
        .active_dialogue
        .as_ref()
        .map(|state| ChoiceUiSnapshot {
            mortar_path: state.mortar_path.clone(),
            node_name: state.current_node.clone(),
            choice_stack: state.choice_stack.clone(),
            choices_broken: state.choices_broken,
        });

    if *last_state == current_state && !button_query.is_empty() {
        return;
    }

    *last_state = current_state.clone();

    for entity in button_query.iter() {
        commands.entity(entity).despawn();
    }

    if let Some(state) = &runtime.active_dialogue
        && let Some(choices) = state.get_choices()
    {
        let should_show_choices = !state.has_next_text_before_choice();
        if !should_show_choices {
            return;
        }

        let font = resources.asset_server.load("font/Unifont.otf");
        let function_decls = resources
            .registry
            .get(&state.mortar_path)
            .and_then(|handle| resources.assets.get(handle))
            .map(|asset| asset.data.functions.as_slice())
            .unwrap_or(&[]);

        for (index, choice) in choices.iter().enumerate() {
            let is_selected = state.selected_choice == Some(index);
            let is_enabled = choice
                .condition
                .as_ref()
                .map(|cond| {
                    bevy_mortar_bond::evaluate_condition(cond, &runtime.functions, function_decls)
                })
                .unwrap_or(true);

            let (bg_color, border_color, text_color) = if !is_enabled {
                (
                    Color::srgb(0.15, 0.15, 0.15),
                    Color::srgb(0.25, 0.25, 0.25),
                    Color::srgb(0.4, 0.4, 0.4),
                )
            } else if is_selected {
                (
                    Color::srgb(0.4, 0.6, 0.2),
                    Color::srgb(0.6, 0.9, 0.3),
                    Color::srgb(1.0, 1.0, 1.0),
                )
            } else {
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

/// Highlights choice buttons when the selection changes.
///
/// 根据选择状态刷新按钮样式。
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
            *bg_color = BackgroundColor(Color::srgb(0.4, 0.6, 0.2));
            *border_color = BorderColor::all(Color::srgb(0.6, 0.9, 0.3));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
            *border_color = BorderColor::all(Color::srgb(0.4, 0.5, 0.65));
        }
    }
}

/// Updates the continue button state/label based on runtime state.
///
/// 根据状态更新“继续”按钮。
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
    runs_executing: Res<MortarRunsExecuting>,
) {
    if !runtime.is_changed() && !runs_executing.is_changed() {
        return;
    }

    for (mut text, mut visibility, mut bg_color, mut border_color) in continue_query.iter_mut() {
        if runs_executing.executing {
            *visibility = Visibility::Visible;
            **text = "执行中...".to_string();
            *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            *border_color = BorderColor::all(Color::srgb(0.25, 0.25, 0.25));
            continue;
        }

        *bg_color = BackgroundColor(Color::srgb(0.2, 0.4, 0.6));
        *border_color = BorderColor::all(Color::srgb(0.4, 0.6, 0.8));

        if let Some(state) = &runtime.active_dialogue {
            if state.has_choices() && !state.has_next_text() {
                if state.selected_choice.is_some() {
                    *visibility = Visibility::Visible;
                    **text = "确认选择".to_string();
                } else {
                    *visibility = Visibility::Hidden;
                }
            } else {
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
/// 处理“重载”按钮点击。
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

            events.write(MortarEvent::StopDialogue);

            let handle = asset_server.load(&path);
            registry.register(path.clone(), handle);

            let start_node = runtime
                .active_dialogue
                .as_ref()
                .map(|state| state.current_node.clone())
                .unwrap_or_else(|| "Start".to_string());

            info!("Example: Restart node {} / {}", &path, &start_node);
            events.write(MortarEvent::StartNode {
                path,
                node: start_node,
            });
        }
    }
}

/// Handles clicks on the "Switch File" button.
///
/// 处理“切换文件”按钮点击。
fn handle_switch_file_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SwitchFileButton>)>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<MortarRegistry>,
    mut events: MessageWriter<MortarEvent>,
    mut dialogue_files: ResMut<DialogueFiles>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            events.write(MortarEvent::StopDialogue);

            dialogue_files.next();
            let path = dialogue_files.current().to_string();
            info!("Example: Switch to file: {}", &path);

            let handle = asset_server.load(&path);
            registry.register(path.clone(), handle);

            const START_NODE: &str = "Start";
            info!("Example: Start a new file node: {} / {}", &path, START_NODE);
            events.write(MortarEvent::StartNode {
                path,
                node: START_NODE.to_string(),
            });
        }
    }
}
