use std::{
    net::UdpSocket,
    time::SystemTime,
};

use super::{PROTOCOL_ID, NetworkPlugin};
use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::{ConnectionConfig, RenetServer}};
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RenetChannelsExt;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{ServerPlugin, ServerSet, ClientConnectedEvent, ClientDisconnectedEvent};
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
pub struct ServerSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkPlugin);

        app.add_event::<ClientConnectedEvent>();
        app.add_event::<ClientDisconnectedEvent>();

        app.add_systems(
            Startup,
            start_server,
        );

        app.add_systems(
            Update,
            (handle_server_events)
                .in_set(ServerSet)
                .run_if(resource_exists::<RenetServer>),
        );
    }
}

fn start_server(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
) {
    let server_channels_config = channels.get_server_configs();
    let client_channels_config = channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let public_addr = "0.0.0.0:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

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
                info!("Player {:?} connected.", client_id);

                connected.send(ClientConnectedEvent { client_id: *client_id });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {:?} disconnected: {}", client_id, reason);

                disconnected.send(ClientDisconnectedEvent {
                    client_id: *client_id,
                    reason: reason.to_string(),
                });
            }
        }
    }
}
