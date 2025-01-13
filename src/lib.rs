//! Tanks library
#![allow(clippy::type_complexity)]

use bevy::prelude::*;

pub mod camera;
pub mod controller;
pub mod main_menu;
pub mod material;
pub mod meth;
pub mod network;
pub mod renderer;
pub mod input;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::GameStates;

    pub use crate::camera::prelude::*;
    pub use crate::controller::prelude::*;
    pub use crate::main_menu::prelude::*;
    pub use crate::material::prelude::*;
    pub use crate::network::prelude::*;
    pub use crate::renderer::prelude::*;
    pub use crate::input::prelude::*;

    #[cfg(feature = "debug")]
    pub use crate::debug::prelude::*;
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    MainMenu,
    Connecting,
    Playing,
}
