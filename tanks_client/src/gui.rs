use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::prelude::*;
use network::prelude::*;
use utils::prelude::*;

pub mod prelude {
    pub use super::GameGuiPlugin;
}

#[derive(Debug, Clone)]
pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInfoMap>();

        app.add_systems(OnEnter(GameStates::Playing), setup_gui);
        app.add_systems(
            Update,
            (handle_player_joined, handle_player_died, handle_player_left)
                .run_if(in_state(GameStates::Playing)),
        );
    }
}

#[derive(Clone, Debug)]
struct PlayerInfo {
    name: String,
}

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
struct PlayerInfoMap(HashMap<ClientId, PlayerInfo>);

#[derive(Component, Clone, Copy, Debug)]
struct GuiChat;

#[derive(Component, Clone, Copy, Debug)]
struct GuiChatEntry;

fn setup_gui(
    mut commands: Commands,
    mut player_info_map: ResMut<PlayerInfoMap>,
    local_player: Res<LocalPlayer>,
    client_info: Res<ClientInfo>,
) {
    player_info_map.insert(
        **local_player,
        PlayerInfo {
            name: client_info.name.clone(),
        },
    );

    commands
        .spawn((
            Name::new("GuiChatRoot"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::End,
                ..default()
            },
            StateScoped(GameStates::Playing),
        ))
        .with_child((
            Name::new("GuiChat"),
            GuiChat,
            Node {
                width: Val::Percent(33.3),
                height: Val::Percent(33.3),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..default()
            },
        ));
}

fn handle_player_joined(
    mut commands: Commands,
    mut joined: EventReader<PlayerJoinedEvent>,
    mut player_info_map: ResMut<PlayerInfoMap>,
    q_chat: Query<Entity, With<GuiChat>>,
) {
    for event in joined.read() {
        if player_info_map.contains_key(&event.client_id) {
            continue;
        }

        if let Ok(entity) = q_chat.get_single() {
            let child = commands
                .spawn((
                    Name::new("GuiChatEntry"),
                    GuiChatEntry,
                    Text::new(format!("{} joined the game", event.name)),
                    DespawnAfter::new(5.0),
                ))
                .id();

            commands.entity(entity).add_child(child);
        }

        player_info_map.insert(
            event.client_id,
            PlayerInfo {
                name: event.name.clone(),
            },
        );
    }
}

fn handle_player_died(
    mut commands: Commands,
    mut died: EventReader<PlayerDiedEvent>,
    player_info_map: Res<PlayerInfoMap>,
    q_chat: Query<Entity, With<GuiChat>>,
) {
    for event in died.read() {
        if let Some(player_info) = player_info_map.get(&event.client_id) {
            if let Ok(entity) = q_chat.get_single() {
                let child = commands
                    .spawn((
                        Name::new("GuiChatEntry"),
                        GuiChatEntry,
                        Text::new(format!("{} exploded", player_info.name)),
                        DespawnAfter::new(5.0),
                    ))
                    .id();

                commands.entity(entity).add_child(child);
            }
        }
    }
}

fn handle_player_left(
    mut commands: Commands,
    mut left: EventReader<PlayerLeftEvent>,
    mut player_info_map: ResMut<PlayerInfoMap>,
    q_chat: Query<Entity, With<GuiChat>>,
) {
    for event in left.read() {
        if let Some(player_info) = player_info_map.remove(&event.client_id) {
            if let Ok(entity) = q_chat.get_single() {
                let child = commands
                    .spawn((
                        Name::new("GuiChatEntry"),
                        GuiChatEntry,
                        Text::new(format!("{} left the game", player_info.name)),
                        DespawnAfter::new(5.0),
                    ))
                    .id();

                commands.entity(entity).add_child(child);
            }
        }
    }
}
