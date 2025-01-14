//! Common stuff for my bevy games.

#![allow(clippy::type_complexity)]

pub mod meth;
pub mod controller;
pub mod material;

pub mod prelude {
    pub use crate::meth::prelude::*;
    pub use crate::controller::prelude::*;
    pub use crate::material::prelude::*;
}
