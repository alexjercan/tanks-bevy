/// Tanks library

pub mod meth;
pub mod camera;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::camera::prelude::*;
}
