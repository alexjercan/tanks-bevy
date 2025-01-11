use std::{collections::HashMap, time::Duration};

use super::EntityKind;
use bevy::prelude::*;
use bevy_renet::renet::*;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{ServerChannel, ServerMessage};
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
    /// The SyncTransform message is used to synchronize the transform of an entity on the client.
    SyncTransform {
        /// The id of the entity.
        id: Entity,
        /// The position of the entity.
        position: Vec3,
        /// The rotation of the entity.
        rotation: Quat,
    },
}

impl From<ServerMessage> for Bytes {
    fn from(message: ServerMessage) -> Self {
        bincode::serialize(&message).unwrap().into()
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
