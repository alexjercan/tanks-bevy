use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::GameGuiPlugin;
}

#[derive(Debug, Clone)]
pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInfoMap>();

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

fn handle_player_joined(
    mut joined: EventReader<PlayerJoinedEvent>,
    mut player_info_map: ResMut<PlayerInfoMap>,
) {
    for event in joined.read() {
        if player_info_map.contains_key(&event.client_id) {
            continue;
        }

        // TODO: show in UI
        info!("Player {} joined", event.name);

        player_info_map.insert(
            event.client_id,
            PlayerInfo {
                name: event.name.clone(),
            },
        );
    }
}

fn handle_player_died(mut died: EventReader<PlayerDiedEvent>, player_info_map: Res<PlayerInfoMap>) {
    for event in died.read() {
        if let Some(player_info) = player_info_map.get(&event.client_id) {
            // TODO: show in UI
            info!("Player {} died", player_info.name);
        }
    }
}

fn handle_player_left(
    mut left: EventReader<PlayerLeftEvent>,
    mut player_info_map: ResMut<PlayerInfoMap>,
) {
    for event in left.read() {
        if let Some(player_info) = player_info_map.remove(&event.client_id) {
            // TODO: show in UI
            info!("Player {} left", player_info.name);
        }
    }
}
