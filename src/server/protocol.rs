use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::network::prelude::{NetworkPlugin, PROTOCOL_ID};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    netcode::{NativeSocket, NetcodeServerTransport, ServerAuthentication, ServerSetupConfig},
    renet2::{ConnectionConfig, RenetServer},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{
        ClientConnectedEvent, ClientDisconnectedEvent, ServerProtocolPlugin, ServerProtocolSet,
    };
}

/// The ClientConnectedEvent is an event that is sent when a client connects to the server
#[derive(Debug, Clone, Event)]
pub struct ClientConnectedEvent {
    pub client_id: ClientId,
}

/// The ClientDisconnectedEvent is an event that is sent when a client disconnects from the server
#[derive(Debug, Clone, Event)]
pub struct ClientDisconnectedEvent {
    pub client_id: ClientId,
    pub reason: String,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerProtocolSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProtocolPlugin;

impl Plugin for ServerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkPlugin);
        app.add_plugins(RepliconRenetPlugins);

        app.add_event::<ClientConnectedEvent>();
        app.add_event::<ClientDisconnectedEvent>();

        app.add_systems(Startup, start_server);

        app.add_systems(
            Update,
            (handle_server_events)
                .in_set(ServerProtocolSet)
                .run_if(resource_exists::<RenetServer>),
        );
    }
}

fn start_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    let server = RenetServer::new(ConnectionConfig::from_channels(
        channels.get_server_configs(),
        channels.get_client_configs(),
    ));

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let public_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5000);
    let socket = UdpSocket::bind(public_addr).unwrap();
    let server_config = ServerSetupConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        socket_addresses: vec![vec![public_addr]],
    };

    let transport =
        NetcodeServerTransport::new(server_config, NativeSocket::new(socket).unwrap()).unwrap();

    commands.insert_resource(server);
    commands.insert_resource(transport);
}

fn handle_server_events(
    mut events: EventReader<ServerEvent>,
    mut connected: EventWriter<ClientConnectedEvent>,
    mut disconnected: EventWriter<ClientDisconnectedEvent>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                debug!("Client {:?} connected.", client_id);

                connected.send(ClientConnectedEvent {
                    client_id: *client_id,
                });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                debug!("Client {:?} disconnected: {}", client_id, reason);

                disconnected.send(ClientDisconnectedEvent {
                    client_id: *client_id,
                    reason: reason.to_string(),
                });
            }
        }
    }
}
