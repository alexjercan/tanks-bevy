//! Tanks library
#![allow(clippy::type_complexity)]

pub mod camera;
pub mod controller;
pub mod main_menu;
pub mod material;
pub mod meth;
pub mod network;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::controller::prelude::*;
    #[cfg(feature = "debug")]
    pub use crate::debug::prelude::*;
    pub use crate::main_menu::prelude::*;
    pub use crate::material::prelude::*;
    pub use crate::network::prelude::*;
}
