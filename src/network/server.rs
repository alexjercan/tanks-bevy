use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, SystemTime},
};

use super::{client::prelude::*, EntityKind, NetworkEntity, PROTOCOL_ID};
use bevy::prelude::*;
use bevy_renet::renet::*;
use bevy_renet::{netcode::*, RenetServerPlugin};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{ServerChannel, ServerMessage, ServerPlugin, ServerSet};
}

#[derive(Resource, Default, Debug)]
struct Lobby {
    names: HashMap<ClientId, String>,
    entities: HashMap<ClientId, Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The ClientConnectedAck message is sent to a client when they connect to the server.
    /// The message informs the client of their own client id.
    ClientConnectedAck {
        /// The id of the client that connected.
        id: ClientId,
    },
    /// The ClientJoined message is sent to all clients when a new client joins the server.
    ClientJoined {
        /// The id of the client that connected.
        id: ClientId,
        /// The name of the client that connected.
        name: String,
    },
    /// The ClientJoinAck message is sent to a client to indicate that they have successfully
    /// joined the lobby.
    ClientJoinAck,
    /// The LobbyInfo message is sent to a client to inform them of the current lobby state.
    LobbyInfo {
        /// The names of the clients in the lobby.
        names: HashMap<ClientId, String>,
    },
    /// The ClientLeft message is sent to all clients when a client leaves the server.
    ClientLeft {
        /// The id of the client that left.
        id: ClientId,
    },
    /// The SpawnEntity message is used to spawn an entity on the client.
    SpawnEntity {
        /// The id of the entity.
        id: Entity,
        /// The position of the entity.
        position: Vec3,
        /// The rotation of the entity.
        rotation: Quat,
        /// The kind of entity to spawn.
        kind: EntityKind,
    },
    /// The DespawnEntity message is used to despawn an entity on the client.
    DespawnEntity {
        /// The id of the entity.
        id: Entity,
    },
}

impl Into<Bytes> for ServerMessage {
    fn into(self) -> Bytes {
        bincode::serialize(&self).unwrap().into()
    }
}

impl From<Bytes> for ServerMessage {
    fn from(bytes: Bytes) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }
}

pub enum ServerChannel {
    Message,
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::Message => 0,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id: Self::Message.into(),
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        }]
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin);
        app.add_plugins(NetcodeServerPlugin);

        let (server, transport) = new_renet_server();
        app.insert_resource(server);
        app.insert_resource(transport);

        app.init_resource::<Lobby>();

        app.add_systems(
            Update,
            (handle_server_events, handle_client_messages)
                .run_if(resource_exists::<RenetServer>)
                .in_set(ServerSet),
        );
    }
}

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "127.0.0.1:5000".parse().unwrap();
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
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

fn handle_server_events(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player {} connected.", client_id);

                let message = ServerMessage::ClientConnectedAck { id: *client_id };
                server.send_message(*client_id, ServerChannel::Message, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {} disconnected: {}", client_id, reason);

                lobby.names.remove(client_id);
                if let Some(id) = lobby.entities.remove(client_id) {
                    let message = ServerMessage::DespawnEntity { id };
                    server.broadcast_message_except(*client_id, ServerChannel::Message, message);

                    commands.entity(id).despawn_recursive();
                }

                let message = ServerMessage::ClientLeft { id: *client_id };
                server.broadcast_message_except(*client_id, ServerChannel::Message, message);
            }
        }
    }
}

fn handle_client_messages(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,
    q_entities: Query<(Entity, &NetworkEntity, &Transform)>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Message) {
            let message: ClientMessage = message.into();
            match message {
                ClientMessage::ClientJoin { name } => {
                    info!("Client {} with name {} joined.", client_id, name);

                    lobby.names.insert(client_id, name.clone());

                    let message = ServerMessage::ClientJoined {
                        id: client_id,
                        name,
                    };
                    server.broadcast_message_except(client_id, ServerChannel::Message, message);
                    let message = ServerMessage::ClientJoinAck;
                    server.send_message(client_id, ServerChannel::Message, message);
                }
                ClientMessage::RequestLobbyInfo => {
                    info!("Client {} requested lobby info.", client_id);

                    let message = ServerMessage::LobbyInfo {
                        names: lobby.names.clone(),
                    };
                    server.send_message(client_id, ServerChannel::Message, message);
                }
                ClientMessage::ClientReady => {
                    info!("Client {} is ready.", client_id);

                    for (entity, network_entity, transform) in q_entities.iter() {
                        let message = ServerMessage::SpawnEntity {
                            id: entity,
                            position: transform.translation,
                            rotation: transform.rotation,
                            kind: network_entity.kind,
                        };
                        server.send_message(client_id, ServerChannel::Message, message);
                    }

                    let position = Vec3::new(
                        rand::random::<f32>() * 20. - 10.,
                        0.5,
                        rand::random::<f32>() * 20. - 10.,
                    );
                    let rotation = Quat::IDENTITY;
                    let id = commands
                        .spawn((
                            Transform::from_translation(position).with_rotation(rotation),
                            NetworkEntity {
                                kind: EntityKind::Tank(client_id),
                            },
                        ))
                        .id();

                    lobby.entities.insert(client_id, id);
                    let message = ServerMessage::SpawnEntity {
                        id,
                        position,
                        rotation,
                        kind: EntityKind::Tank(client_id),
                    };
                    server.broadcast_message(ServerChannel::Message, message);
                }
                ClientMessage::ControllerInput { forward, steer } => {

                },
            }
        }
    }
}
