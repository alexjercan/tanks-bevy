use bevy::prelude::*;
use tanks::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(ClientPlugin);
    app.run();
}
