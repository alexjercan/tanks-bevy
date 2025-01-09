/// Tanks library

pub mod meth;
pub mod camera;
pub mod material;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::material::prelude::*;
}
