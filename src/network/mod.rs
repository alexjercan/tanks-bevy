pub mod client;
pub mod server;

use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

pub mod prelude {
    pub use super::client::prelude::*;
    pub use super::server::prelude::*;
    pub use super::{NetworkEntity, PROTOCOL_ID, Ground, Player};
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NetworkEntity;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Ground {
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Player {
    pub client_id: ClientId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins);
        app.add_plugins(RepliconRenetPlugins);

        app.replicate::<NetworkEntity>();
        app.replicate::<Ground>();
        app.replicate::<Player>();
        app.replicate_group::<(Transform, NetworkEntity)>(); // NetworkTransform
    }
}
