use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    renet2::{ConnectionConfig, RenetClient},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

use network::prelude::*;

#[cfg(not(target_family = "wasm"))]
use protocol_native::create_client;

#[cfg(target_family = "wasm")]
use protocol_wasm::create_client;

pub mod prelude {
    pub use super::{
        ClientConnectEvent, ClientProtocolPlugin, ClientProtocolSet, LocalPlayer, LocalPlayerEntity,
    };
}

/// The ClientConnectEvent is an event that is sent when the client wants to connect to a server
/// with the given address.
#[derive(Debug, Clone, Event)]
pub struct ClientConnectEvent {
    pub address: String,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct LocalPlayer(pub ClientId);

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct LocalPlayerEntity(pub Entity);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientProtocolSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProtocolPlugin;

impl Plugin for ClientProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkPlugin);
        app.add_plugins(RepliconRenetPlugins);

        app.add_event::<ClientConnectEvent>();

        app.add_systems(
            Update,
            (handle_client_connect)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<RenetClient>)),
        );
        app.add_systems(
            Update,
            (update_local_player_entity)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<LocalPlayerEntity>))
                .run_if(resource_exists::<LocalPlayer>),
        );
    }
}

pub fn handle_client_connect(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut connect_events: EventReader<ClientConnectEvent>,
) {
    for ClientConnectEvent { address } in connect_events.read() {
        let config = ConnectionConfig::from_channels(
            channels.get_server_configs(),
            channels.get_client_configs(),
        );

        let (client, transport) = create_client(address.clone(), config, PROTOCOL_ID);

        commands.insert_resource(LocalPlayer(ClientId::new(transport.client_id())));
        commands.insert_resource(client);
        commands.insert_resource(transport);
    }
}

fn update_local_player_entity(
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    q_player: Query<(Entity, &Player)>,
) {
    for (entity, player) in q_player.iter() {
        if player.client_id == **local_player {
            commands.insert_resource(LocalPlayerEntity(entity));
        }
    }
}
