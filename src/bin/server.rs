use bevy::prelude::*;
use tanks::server::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(ServerPlugin);
    app.run();
}