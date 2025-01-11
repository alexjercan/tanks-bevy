use std::time::Duration;

use bevy_renet::renet::*;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{ClientChannel, ClientMessage};
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

impl From<ClientMessage> for Bytes {
    fn from(message: ClientMessage) -> Self {
        bincode::serialize(&message).unwrap().into()
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
