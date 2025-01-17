//! This module contains the client-side game logic.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[cfg(feature = "server")]
pub mod tanks_server;

#[cfg(feature = "client")]
pub mod tanks_client;

pub mod network;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    MainMenu,
    Connecting,
    Playing,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    pub tank: Handle<Scene>,
    #[asset(path = "models/shell.glb#Scene0")]
    pub shell: Handle<Scene>,
    #[asset(
        paths(
            "prototype/prototype-aqua.png",
            "prototype/prototype-orange.png",
            "prototype/prototype-yellow.png",
            "prototype/prototype-blue.png",
            "prototype/prototype-purple.png",
            "prototype/prototype-green.png",
            "prototype/prototype-red.png",
        ),
        collection(typed)
    )]
    pub prototype_textures: Vec<Handle<Image>>,
    #[asset(path = "sounds/cannon_fire.ogg")]
    pub cannon_fire: Handle<bevy_kira_audio::AudioSource>,
    #[asset(path = "sounds/shell_impact.ogg")]
    pub shell_impact: Handle<bevy_kira_audio::AudioSource>,
    #[asset(path = "sounds/death.ogg")]
    pub death: Handle<bevy_kira_audio::AudioSource>,
    #[asset(path = "sounds/tank_engine.ogg")]
    pub tank_engine: Handle<bevy_kira_audio::AudioSource>,
}

pub mod prelude {
    #[cfg(feature = "server")]
    pub use super::tanks_server::prelude::*;

    #[cfg(feature = "client")]
    pub use super::tanks_client::prelude::*;

    pub use super::network::prelude::*;

    pub use super::GameAssets;
    pub use super::GameStates;
}
