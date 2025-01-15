//! This module contains utilities for working with moving transforms.

pub mod orbiter;
pub mod smooth;
pub mod tank;

pub mod prelude {
    pub use super::orbiter::prelude::*;
    pub use super::smooth::prelude::*;
    pub use super::tank::prelude::*;
}
