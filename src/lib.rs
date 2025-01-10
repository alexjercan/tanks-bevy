/// Tanks library

pub mod meth;
pub mod camera;
pub mod material;
pub mod controller;
pub mod network;
pub mod main_menu;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::material::prelude::*;
    pub use crate::controller::prelude::*;
    pub use crate::network::prelude::*;
    pub use crate::main_menu::prelude::*;
    #[cfg(feature = "debug")]
    pub use crate::debug::prelude::*;
}
