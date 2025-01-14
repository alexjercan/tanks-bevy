//! This module contains the server-side game logic.

pub mod protocol;
pub mod server;
pub mod cannon;

pub mod prelude {
    pub use super::protocol::prelude::*;
    pub use super::server::prelude::*;
    pub use super::cannon::prelude::*;
}
