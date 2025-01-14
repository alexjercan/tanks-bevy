use bevy::{prelude::*, ui::FocusPolicy};
use bevy_simple_text_input::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{ClientInfo, MainMenuPlugin, PlayButtonPressed};
}

#[derive(Resource, Debug, Clone)]
pub struct ClientInfo {
    pub address: String,
    pub name: String,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            name: "Player".to_string(),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct MainMenu;

#[derive(Component, Clone, Copy, Debug)]
struct AddressInput;

#[derive(Component, Clone, Copy, Debug)]
struct NameInput;

#[derive(Component, Clone, Copy, Debug)]
struct PlayButton;

#[derive(Debug, Clone, Event)]
pub struct PlayButtonPressed {
    pub address: String,
    pub name: String,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TextInputPlugin)
            .init_resource::<ClientInfo>()
            .add_event::<PlayButtonPressed>()
            .add_systems(OnEnter(GameStates::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                (
                    interact_with_play_button,
                    focus_text_input.before(TextInputSystem),
                )
                    .run_if(in_state(GameStates::MainMenu))
                    .chain(),
            );
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(GameStates::MainMenu),
    ));

    commands
        .spawn((
            Name::new("MainMenu"),
            MainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(GameStates::MainMenu),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("PlayButton"),
                    PlayButton,
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Play"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));

            parent.spawn((
                Name::new("AddressInput"),
                AddressInput,
                Node {
                    width: Val::Px(500.0),
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Interaction::None,
                BorderColor(BORDER_COLOR_INACTIVE),
                BackgroundColor(BACKGROUND_COLOR),
                FocusPolicy::Block,
                TextInput,
                TextInputTextFont(TextFont {
                    font_size: 34.,
                    ..default()
                }),
                TextInputTextColor(TextColor(TEXT_COLOR)),
                TextInputValue("127.0.0.1".to_string()),
                TextInputSettings {
                    retain_on_submit: true,
                    ..default()
                },
                TextInputInactive(true),
            ));

            parent.spawn((
                Name::new("NameInput"),
                NameInput,
                Node {
                    width: Val::Px(200.0),
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Interaction::None,
                BorderColor(BORDER_COLOR_INACTIVE),
                BackgroundColor(BACKGROUND_COLOR),
                FocusPolicy::Block,
                TextInput,
                TextInputTextFont(TextFont {
                    font_size: 34.,
                    ..default()
                }),
                TextInputTextColor(TextColor(TEXT_COLOR)),
                TextInputValue("Player".to_string()),
                TextInputSettings {
                    retain_on_submit: true,
                    ..default()
                },
                TextInputInactive(true),
            ));
        });
}

fn interact_with_play_button(
    mut q_interaction: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<PlayButton>),
    >,
    q_address: Query<&TextInputValue, With<AddressInput>>,
    q_name: Query<&TextInputValue, With<NameInput>>,
    mut events: EventWriter<PlayButtonPressed>,
) {
    for (interaction, mut color) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let address = q_address.get_single().expect("AddressInput not found");
                let name = q_name.get_single().expect("NameInput not found");

                events.send(PlayButtonPressed {
                    address: address.0.clone(),
                    name: name.0.clone(),
                });
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn focus_text_input(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, mut border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                    *border_color = BORDER_COLOR_ACTIVE.into();
                } else {
                    inactive.0 = true;
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }
}
