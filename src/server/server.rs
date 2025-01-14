use std::{collections::HashMap, time::Duration};

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    winit::WinitPlugin,
};
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use crate::server::prelude::*;
use crate::network::prelude::*;
use utils::prelude::*;

pub mod prelude {
    pub use super::ServerPlugin;
}

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct ClientMap(HashMap<ClientId, Entity>);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
                    ..default()
                }),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ));
        app.add_plugins(ServerProtocolPlugin);
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(TankControllerPlugin);

        app.init_resource::<ClientMap>();

        app.add_systems(Startup, setup_game);
        app.add_systems(
            Update,
            (
                handle_client_connected,
                handle_client_disconnected,
                handle_player_join,
                handle_player_input,
            ),
        );
    }
}

fn setup_game(mut commands: Commands) {
    let size = 100.0;

    commands.spawn((
        Name::new("Ground"),
        Replicated,
        Transform::default(),
        NetworkEntity,
        Ground {
            width: size,
            height: size,
        },
        Collider::cuboid(size / 2.0, f32::EPSILON, size / 2.0),
    ));
}

fn handle_player_join(
    mut commands: Commands,
    mut join: EventReader<FromClient<PlayerJoinEvent>>,
    mut client_map: ResMut<ClientMap>,
) {
    for FromClient { client_id, event } in join.read() {
        let position = Vec3::new(
            rand::random::<f32>() * 20. - 10.,
            0.5,
            rand::random::<f32>() * 20. - 10.,
        );
        let rotation = Quat::IDENTITY;

        let entity = commands
            .spawn((
                Replicated,
                Name::new("Player"),
                Transform::from_translation(position).with_rotation(rotation),
                NetworkEntity,
                Player {
                    client_id: *client_id,
                    name: event.name.clone(),
                    color: event.color,
                },
                Collider::cuboid(0.4, 0.2, 0.4),
                KinematicCharacterController {
                    custom_mass: Some(5.0),
                    up: Vec3::Y,
                    offset: CharacterLength::Absolute(0.01),
                    slide: true,
                    autostep: Some(CharacterAutostep {
                        max_height: CharacterLength::Relative(0.3),
                        min_width: CharacterLength::Relative(0.5),
                        include_dynamic_bodies: false,
                    }),
                    // Don’t allow climbing slopes larger than 45 degrees.
                    max_slope_climb_angle: 45.0_f32.to_radians(),
                    // Automatically slide down on slopes smaller than 30 degrees.
                    min_slope_slide_angle: 30.0_f32.to_radians(),
                    apply_impulse_to_dynamic_bodies: true,
                    snap_to_ground: None,
                    ..default()
                },
                TankControllerInput::default(),
                TankController::default(),
            ))
            .id();

        client_map.insert(*client_id, entity);
    }
}

fn handle_player_input(
    mut input: EventReader<FromClient<PlayerInputEvent>>,
    mut q_players: Query<&mut TankControllerInput>,
    client_map: ResMut<ClientMap>,
) {
    for FromClient { client_id, event } in input.read() {
        if let Some(entity) = client_map.get(client_id) {
            if let Ok(mut player_input) = q_players.get_mut(*entity) {
                player_input.forward = event.y;
                player_input.steer = event.x;
            }
        }
    }
}

fn handle_client_disconnected(
    mut commands: Commands,
    mut disconnected: EventReader<ClientDisconnectedEvent>,
    mut client_map: ResMut<ClientMap>,
) {
    for ClientDisconnectedEvent {
        client_id,
        reason: _,
    } in disconnected.read()
    {
        if let Some(entity) = client_map.remove(client_id) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_client_connected(mut connected: EventReader<ClientConnectedEvent>) {
    for ClientConnectedEvent { .. } in connected.read() {
        // TODO: some stuff when a client connects
    }
}
