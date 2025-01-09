/// Tanks library

pub mod meth;
pub mod camera;
pub mod material;
pub mod controller;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::material::prelude::*;
    pub use crate::controller::prelude::*;
    #[cfg(feature = "debug")]
    pub use crate::debug::prelude::*;
}
