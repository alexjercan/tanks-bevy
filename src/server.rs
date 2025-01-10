use std::{f32::EPSILON, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, prelude::*, winit::WinitPlugin};
use bevy_rapier3d::prelude::*;
use tanks::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
    app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)));

    // Network
    app.add_plugins(ServerPlugin);

    // Server Side
    app.add_systems(Startup, setup_game);

    app.run();
}

fn setup_game(mut commands: Commands) {
    // Ground
    let size = 32;
    commands.spawn((
        Transform::from_translation(Vec3::ZERO),
        Collider::cuboid(size as f32 / 2.0, EPSILON, size as f32 / 2.0),
    ));
}
