//! Common stuff for my bevy games.

#![allow(clippy::type_complexity)]

pub mod controller;
pub mod health;
pub mod material;
pub mod meth;
pub mod physics;
pub mod despawn;

pub mod prelude {
    pub use crate::controller::prelude::*;
    pub use crate::health::prelude::*;
    pub use crate::material::prelude::*;
    pub use crate::meth::prelude::*;
    pub use crate::physics::prelude::*;
    pub use crate::despawn::prelude::*;
}
