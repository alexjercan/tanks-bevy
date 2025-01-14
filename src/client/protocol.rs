use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::network::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    netcode::{ClientAuthentication, NativeSocket, NetcodeClientTransport},
    renet2::{ConnectionConfig, RenetClient},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

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

fn handle_client_connect(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut connect_events: EventReader<ClientConnectEvent>,
) {
    for ClientConnectEvent { address } in connect_events.read() {
        let client = RenetClient::new(
            ConnectionConfig::from_channels(
                channels.get_server_configs(),
                channels.get_client_configs(),
            ),
            false,
        );

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        let server_addr = address.parse().unwrap();
        let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)).unwrap();
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            socket_id: 0,
            server_addr,
            user_data: None,
        };

        let transport = NetcodeClientTransport::new(
            current_time,
            authentication,
            NativeSocket::new(socket).unwrap(),
        )
        .unwrap();

        commands.insert_resource(LocalPlayer(ClientId::new(client_id)));
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
