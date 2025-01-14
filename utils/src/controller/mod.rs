//! This module contains utilities for working with moving transforms.

pub mod tank;
pub mod orbiter;
pub mod smooth;

pub mod prelude {
    pub use super::tank::prelude::*;
    pub use super::orbiter::prelude::*;
    pub use super::smooth::prelude::*;
}
