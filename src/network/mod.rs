pub mod client;
pub mod server;

use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

pub mod prelude {
    pub use super::client::prelude::*;
    pub use super::server::prelude::*;
    pub use super::{Ground, NetworkEntity, Player, PlayerInputEvent, PROTOCOL_ID};
    pub use bevy_replicon::prelude::client_connected;
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

#[derive(Debug, Default, Deserialize, Event, Serialize, Deref, DerefMut)]
pub struct PlayerInputEvent(pub Vec2);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins);
        app.add_plugins(RepliconRenetPlugins);

        app.add_client_event::<PlayerInputEvent>(ChannelKind::Ordered);

        app.replicate::<Name>();
        app.replicate::<NetworkEntity>();
        app.replicate::<Ground>();
        app.replicate::<Player>();
        app.replicate_group::<(Transform, NetworkEntity)>(); // NetworkTransform
    }
}
