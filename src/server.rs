use bevy::{log::{Level, LogPlugin}, prelude::*, state::app::StatesPlugin};
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use tanks::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        LogPlugin {
            level: Level::INFO,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
            ..default()
        },
        ServerPlugin,
    ));

    app.add_systems(Startup, setup_game);
    app.add_systems(Update, handle_client_connected);

    app.run();
}

fn setup_game(mut commands: Commands) {
    let size = 100.0;

    commands.spawn((
        Replicated,
        Transform::default(),
        NetworkEntity,
        Ground { width: size, height: size },
        Collider::cuboid(size / 2.0, f32::EPSILON, size / 2.0),
    ));
}

fn handle_client_connected(
    mut commands: Commands,
    mut connected: EventReader<ClientConnectedEvent>,
) {
    for ClientConnectedEvent { client_id } in connected.read() {
        let position = Vec3::new(
            rand::random::<f32>() * 20. - 10.,
            0.5,
            rand::random::<f32>() * 20. - 10.,
        );
        let rotation = Quat::IDENTITY;

        commands.spawn((
            Replicated,
            Transform::from_translation(position).with_rotation(rotation),
            NetworkEntity,
            Player { client_id: *client_id },
        ));
    }
}
