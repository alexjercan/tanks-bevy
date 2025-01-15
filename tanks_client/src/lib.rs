//! This module contains the client-side game logic.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod camera;
pub mod client;
pub mod gui;
pub mod input;
pub mod main_menu;
pub mod protocol;
pub mod renderer;
pub mod audio;
pub mod particles;

#[cfg(feature = "debug")]
pub mod debug;

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
}

pub mod prelude {
    pub use super::camera::prelude::*;
    pub use super::client::prelude::*;
    pub use super::gui::prelude::*;
    pub use super::input::prelude::*;
    pub use super::main_menu::prelude::*;
    pub use super::protocol::prelude::*;
    pub use super::renderer::prelude::*;
    pub use super::audio::prelude::*;
    pub use super::particles::prelude::*;

    #[cfg(feature = "debug")]
    pub use super::debug::prelude::*;

    pub use super::GameStates;
    pub use super::GameAssets;
}
