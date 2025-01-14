use bevy::prelude::*;
use tanks_server::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(ServerPlugin);
    app.run();
}
