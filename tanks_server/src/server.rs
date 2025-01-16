use std::{collections::HashMap, time::Duration};

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    winit::WinitPlugin,
};
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use utils::prelude::*;

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::ServerPlugin;
}

#[derive(Clone, Debug)]
struct PlayerInfo {
    name: String,
    color: Color,
}

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
struct PlayerInfoMap(HashMap<ClientId, PlayerInfo>);

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
struct PlayerEntityMap(HashMap<ClientId, Entity>);

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
        app.add_plugins(CollisionPlugin);
        app.add_plugins(TankControllerPlugin);
        app.add_plugins(TankCannonPlugin);
        app.add_plugins(HealthPlugin);

        app.init_resource::<PlayerInfoMap>();
        app.init_resource::<PlayerEntityMap>();

        app.add_systems(Startup, setup_game);
        app.add_systems(
            Update,
            (
                handle_client_connected,
                handle_client_disconnected,
                handle_player_join,
                handle_player_spawn,
                handle_player_input,
                handle_player_fire,
                handle_player_dead,
                handle_player_throttle,
            ),
        );
    }
}

fn spawn_player(commands: &mut Commands, client_id: &ClientId, info: &PlayerInfo) -> Entity {
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
                name: info.name.clone(),
                color: info.color,
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
            TankCannonInput::default(),
            TankCannon::default(),
            Health::default(),
            Throttle { value: 0.0 },
        ))
        .id();

    entity
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

fn handle_client_connected(
    mut connected: EventReader<ClientConnectedEvent>,
    mut joined: EventWriter<ToClients<PlayerJoinedEvent>>,
    player_info_map: Res<PlayerInfoMap>,
) {
    for ClientConnectedEvent { client_id } in connected.read() {
        for (id, info) in player_info_map.iter() {
            joined.send(ToClients {
                mode: SendMode::Direct(*client_id),
                event: PlayerJoinedEvent {
                    client_id: *id,
                    name: info.name.clone(),
                },
            });
        }
    }
}

fn handle_client_disconnected(
    mut commands: Commands,
    mut disconnected: EventReader<ClientDisconnectedEvent>,
    mut player_entity_map: ResMut<PlayerEntityMap>,
    mut player_info_map: ResMut<PlayerInfoMap>,
    mut left: EventWriter<ToClients<PlayerLeftEvent>>,
) {
    for ClientDisconnectedEvent {
        client_id,
        reason: _,
    } in disconnected.read()
    {
        if let Some(entity) = player_entity_map.remove(client_id) {
            commands.entity(entity).despawn_recursive();
        }

        if let Some(player_info) = player_info_map.remove(client_id) {
            info!("Player {} disconnected", player_info.name);

            left.send(ToClients {
                mode: SendMode::BroadcastExcept(*client_id),
                event: PlayerLeftEvent {
                    client_id: *client_id,
                },
            });
        }
    }
}

fn handle_player_join(
    mut join: EventReader<FromClient<PlayerJoinEvent>>,
    mut joined: EventWriter<ToClients<PlayerJoinedEvent>>,
    mut player_info_map: ResMut<PlayerInfoMap>,
) {
    for FromClient { client_id, event } in join.read() {
        if player_info_map.contains_key(client_id) {
            continue;
        }

        info!("Player {} joined", event.name);

        player_info_map.insert(
            *client_id,
            PlayerInfo {
                name: event.name.clone(),
                color: event.color,
            },
        );

        joined.send(ToClients {
            mode: SendMode::BroadcastExcept(*client_id),
            event: PlayerJoinedEvent {
                client_id: *client_id,
                name: event.name.clone(),
            },
        });
    }
}

fn handle_player_spawn(
    mut commands: Commands,
    mut spawn: EventReader<FromClient<PlayerSpawnEvent>>,
    mut player_entity_map: ResMut<PlayerEntityMap>,
    player_info_map: Res<PlayerInfoMap>,
) {
    for FromClient { client_id, .. } in spawn.read() {
        if player_entity_map.contains_key(client_id) {
            continue;
        }

        if let Some(player_info) = player_info_map.get(client_id) {
            info!("Player {} spawned", player_info.name);

            let entity = spawn_player(&mut commands, client_id, player_info);

            player_entity_map.insert(*client_id, entity);
        }
    }
}

fn handle_player_input(
    mut input: EventReader<FromClient<PlayerInputEvent>>,
    mut q_player: Query<&mut TankControllerInput>,
    player_entity_map: Res<PlayerEntityMap>,
) {
    for FromClient { client_id, event } in input.read() {
        if let Some(entity) = player_entity_map.get(client_id) {
            if let Ok(mut player_input) = q_player.get_mut(*entity) {
                player_input.forward = event.y;
                player_input.steer = event.x;
            }
        }
    }
}

fn handle_player_fire(
    mut fire: EventReader<FromClient<PlayerFireEvent>>,
    mut q_player: Query<&mut TankCannonInput>,
    player_entity_map: Res<PlayerEntityMap>,
) {
    for FromClient { client_id, .. } in fire.read() {
        if let Some(entity) = player_entity_map.get(client_id) {
            if let Ok(mut player_fire) = q_player.get_mut(*entity) {
                player_fire.fire = true;
            }
        }
    }
}

fn handle_player_dead(
    mut commands: Commands,
    q_player: Query<(Entity, &Transform, &Player), With<Dead>>,
    mut player_entity_map: ResMut<PlayerEntityMap>,
    mut died: EventWriter<ToClients<PlayerDiedEvent>>,
) {
    for (
        entity,
        transform,
        Player {
            client_id, name, ..
        },
    ) in q_player.iter()
    {
        println!("Player {} is dead", name);

        player_entity_map.remove(client_id);

        commands.entity(entity).despawn_recursive();

        died.send(ToClients {
            mode: SendMode::Broadcast,
            event: PlayerDiedEvent {
                client_id: *client_id,
                position: transform.translation,
            },
        });
    }
}

fn handle_player_throttle(
    mut q_player: Query<(&TankControllerInput, &mut Throttle)>,
) {
    for (input, mut throttle) in q_player.iter_mut() {
        throttle.value = input.forward.abs();
    }
}
