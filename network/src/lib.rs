use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        Ground, NetworkEntity, NetworkPlugin, Player, PlayerDiedEvent, PlayerFireEvent,
        PlayerInputEvent, PlayerJoinEvent, PlayerJoinedEvent, PlayerLeftEvent, PlayerSpawnEvent,
        Shell, PROTOCOL_ID,
    };
    pub use bevy_replicon::prelude::{client_connected, client_just_connected};
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NetworkEntity;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Ground {
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub client_id: ClientId,
    pub name: String,
    pub color: Color,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Shell;

#[derive(Debug, Default, Deserialize, Event, Serialize, Deref, DerefMut)]
pub struct PlayerInputEvent(pub Vec2);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct PlayerFireEvent;

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct PlayerJoinEvent {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct PlayerSpawnEvent;

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct PlayerJoinedEvent {
    pub client_id: ClientId,
    pub name: String,
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct PlayerLeftEvent {
    pub client_id: ClientId,
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct PlayerDiedEvent {
    pub client_id: ClientId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins);

        app.add_client_event::<PlayerInputEvent>(ChannelKind::Ordered);
        app.add_client_event::<PlayerFireEvent>(ChannelKind::Ordered);
        app.add_client_event::<PlayerJoinEvent>(ChannelKind::Ordered);
        app.add_client_event::<PlayerSpawnEvent>(ChannelKind::Ordered);

        app.add_server_event::<PlayerJoinedEvent>(ChannelKind::Ordered);
        app.add_server_event::<PlayerDiedEvent>(ChannelKind::Ordered);
        app.add_server_event::<PlayerLeftEvent>(ChannelKind::Ordered);

        app.replicate::<Name>();
        app.replicate::<NetworkEntity>();
        app.replicate::<Ground>();
        app.replicate::<Player>();
        app.replicate::<Shell>();
        app.replicate_group::<(Transform, NetworkEntity)>(); // NetworkTransform
    }
}
