//! This module contains the client-side game logic.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

pub mod camera;
pub mod client;
pub mod gui;
pub mod input;
pub mod main_menu;
pub mod protocol;
pub mod renderer;

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

pub mod prelude {
    pub use super::camera::prelude::*;
    pub use super::client::prelude::*;
    pub use super::gui::prelude::*;
    pub use super::input::prelude::*;
    pub use super::main_menu::prelude::*;
    pub use super::protocol::prelude::*;
    pub use super::renderer::prelude::*;

    #[cfg(feature = "debug")]
    pub use super::debug::prelude::*;

    pub use super::GameStates;
}
