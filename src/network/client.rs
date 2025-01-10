use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, SystemTime},
};

use bevy::prelude::*;
use bevy_renet::{client_connected, netcode::*, renet::*, RenetClientPlugin};
use serde::{Deserialize, Serialize};

use super::{server::prelude::*, NetworkEntity, PROTOCOL_ID};

pub mod prelude {
    pub use super::{
        ClientChannel, ClientInfo, ClientMessage, ClientPlugin, ClientSet, LocalPlayer,
    };
}

#[derive(Resource, Default, Debug)]
struct Lobby {
    /// The names of the clients in the lobby.
    names: HashMap<ClientId, String>,
    /// Map from server entity id to client entity id.
    entities: HashMap<Entity, Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The ClientMessage enum represents the messages that can be sent from the client to the server.
pub enum ClientMessage {
    /// The ClientJoin message is sent to the server to inform that the client wants to join the
    /// lobby.
    ClientJoin {
        /// The name of the client.
        name: String,
    },
    /// The RequestLobbyInfo message is sent to the server to request the lobby information.
    RequestLobbyInfo,
    /// The ClientReady message is sent to the server to inform that the client is ready to start
    /// playing.
    ClientReady,
    /// The ControllerInput message is sent to the server to inform that the client has input
    /// for their character controller.
    ControllerInput {
        /// The forward input.
        forward: f32,
        /// The steer input.
        steer: f32,
    },
}

impl Into<Bytes> for ClientMessage {
    fn into(self) -> Bytes {
        bincode::serialize(&self).unwrap().into()
    }
}

impl From<Bytes> for ClientMessage {
    fn from(bytes: Bytes) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }
}

/// TODO: Split messages into multiple types with different channels, e.g: Input, Command, etc.
pub enum ClientChannel {
    Message,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Message => 0,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id: Self::Message.into(),
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::ZERO,
            },
        }]
    }
}

#[derive(Resource, Default, Debug)]
pub struct TankControllerInput {
    pub forward: f32,
    pub steer: f32,
}

#[derive(Resource, Default, Debug)]
pub struct LocalPlayer {
    pub id: Option<ClientId>,
}

#[derive(Resource, Debug)]
pub struct ClientInfo {
    pub address: String,
    pub name: String,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:5000".to_string(),
            name: "Player".to_string(),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(NetcodeClientPlugin);
        app.init_resource::<ClientInfo>();
        app.init_resource::<LocalPlayer>();
        app.init_resource::<Lobby>();
        app.init_resource::<TankControllerInput>();

        app.add_systems(
            Update,
            setup_network
                .in_set(ClientSet)
                .run_if(not(resource_exists::<RenetClient>)),
        );
        app.add_systems(
            Update,
            handle_server_messages
                .in_set(ClientSet)
                .run_if(client_connected),
        );
    }
}

fn setup_network(mut commands: Commands, client_info: Res<ClientInfo>) {
    let server_addr = client_info.address.parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    commands.insert_resource(client);
    commands.insert_resource(transport);
}

fn handle_server_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut local_player: ResMut<LocalPlayer>,
    mut lobby: ResMut<Lobby>,
    client_info: Res<ClientInfo>,
) {
    while let Some(message) = client.receive_message(ServerChannel::Message) {
        let message: ServerMessage = message.into();
        match message {
            ServerMessage::ClientConnectedAck { id } => {
                info!("Connected to server with id: {}", id);
                local_player.id = Some(id);

                let message = ClientMessage::ClientJoin {
                    name: client_info.name.clone(),
                };
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::ClientJoined { id, name } => {
                info!("Player {} joined with name: {}", id, name);

                lobby.names.insert(id, name.clone());
            }
            ServerMessage::ClientJoinAck => {
                info!("Successfully joined the lobby.");

                let message = ClientMessage::RequestLobbyInfo;
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::LobbyInfo { names } => {
                info!("Received lobby info: {:?}", names);

                lobby.names = names;

                let message = ClientMessage::ClientReady;
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::ClientLeft { id } => {
                info!("Player {} left the lobby.", id);

                lobby.names.remove(&id);
            }
            ServerMessage::SpawnEntity {
                id,
                position,
                rotation,
                kind,
            } => {
                info!(
                    "Spawning entity {} ({:?}) at {:?} with rotation {:?}.",
                    id, kind, position, rotation
                );

                let local_id = commands
                    .spawn((
                        Transform::from_translation(position).with_rotation(rotation),
                        Visibility::default(),
                        NetworkEntity { kind },
                    ))
                    .id();

                lobby.entities.insert(id, local_id);
            }
            ServerMessage::DespawnEntity { id } => {
                info!("Despawning entity {}.", id);

                if let Some(local_id) = lobby.entities.remove(&id) {
                    commands.entity(local_id).despawn_recursive();
                }
            }
        }
    }
}
