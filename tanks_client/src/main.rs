use bevy::prelude::*;
use tanks_client::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(ClientPlugin);
    app.run();
}
