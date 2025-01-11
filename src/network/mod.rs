pub mod client;
pub mod server;

use bevy_renet::renet::*;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

pub mod prelude {
    pub use super::client::prelude::*;
    pub use super::server::prelude::*;
    pub use super::{EntityKind, NetworkEntity, PROTOCOL_ID};
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityKind {
    Tank(ClientId),
}

#[derive(Component, Clone, Copy, Debug)]
pub struct NetworkEntity {
    pub kind: EntityKind,
}
