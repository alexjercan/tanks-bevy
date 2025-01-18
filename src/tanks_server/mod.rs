//! This module contains the server-side game logic.

pub mod cannon;
pub mod protocol;
pub mod server;

pub mod prelude {
    pub use super::cannon::prelude::*;
    pub use super::protocol::prelude::*;
    pub use super::server::prelude::*;
}
