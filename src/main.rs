use std::f32::{
    consts::{FRAC_PI_2, PI},
    EPSILON,
};

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use tanks::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    tank: Handle<Scene>,
    #[asset(path = "models/shell.glb#Scene0")]
    shell: Handle<Scene>,
    #[asset(
        paths(
            "prototype/prototype-aqua.png",
            "prototype/prototype-orange.png",
            "prototype/prototype-yellow.png",
            "prototype/prototype-blue.png",
            "prototype/prototype-purple.png",
            "prototype/prototype-green.png",
            "prototype/prototype-red.png",
        ),
        collection(typed)
    )]
    pub prototype_textures: Vec<Handle<Image>>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Damage {
    pub amount: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TankCannon {
    /// The fire rate of the cannon (in seconds per shot)
    pub fire_rate_secs: f32,
    /// The speed of the shell
    pub shell_speed: f32,
    /// The offset of the cannon from the tank
    pub offset: Vec3,
}

impl Default for TankCannon {
    fn default() -> Self {
        Self {
            fire_rate_secs: 1.0,
            shell_speed: 25.0,
            offset: Vec3::new(0.0, 0.23, 0.6),
        }
    }
}

#[derive(Component, Clone, Debug)]
struct TankCannonState {
    /// The cooldown time remaining before we can fire again (in seconds)
    cooldown: Timer,
}

#[derive(Resource, Default, Debug)]
pub struct TankCannonInput {
    pub fire: bool,
}

impl Default for TankCannonState {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TankCannonSet;

pub struct TankCannonPlugin;

impl Plugin for TankCannonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TankCannonInput>().add_systems(
            Update,
            (initialize_cannon_state, initialize_cannon_shell_state, cannon_fire, shell_update_time_to_live, shell_update_collision)
                .in_set(TankCannonSet)
                .chain(),
        );
    }
}

fn initialize_cannon_state(
    mut commands: Commands,
    q_cannon: Query<(Entity, &TankCannon), Without<TankCannonState>>,
) {
    for (entity, _cannon) in q_cannon.iter() {
        commands
            .entity(entity)
            .insert(TankCannonState { ..default() });
    }
}

fn initialize_cannon_shell_state(
    mut commands: Commands,
    q_shell: Query<(Entity, &TankCannonShell), Without<TankCannonShellState>>,
) {
    for (entity, shell) in q_shell.iter() {
        commands
            .entity(entity)
            .insert(TankCannonShellState {
                time_to_live: Timer::from_seconds(shell.time_to_live, TimerMode::Once),
            });
    }
}

fn shell_update_time_to_live(
    time: Res<Time>,
    mut commands: Commands,
    mut q_shell: Query<(Entity, &TankCannonShell, &mut TankCannonShellState)>,
) {
    for (entity, _shell, mut state) in q_shell.iter_mut() {
        if state.time_to_live.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn shell_update_collision(
    mut commands: Commands,
    q_shell: Query<(Entity, &TankCannonShell, &CollisionWith)>,
) {
    for (entity, shell, collision_with) in q_shell.iter() {
        commands.entity(entity).despawn_recursive();
        commands.entity(collision_with.entity).insert(Damage { amount: shell.damage });
    }
}

fn cannon_fire(
    time: Res<Time>,
    mut commands: Commands,
    input: Res<TankCannonInput>,
    mut q_cannon: Query<(&Transform, &TankCannon, &mut TankCannonState)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    for (transform, cannon, mut state) in q_cannon.iter_mut() {
        if state.cooldown.tick(time.delta()).finished() {
            if !input.fire {
                continue;
            }

            commands
                .spawn((
                    Transform::from_translation(
                        transform.translation + transform.rotation * cannon.offset,
                    )
                    .with_rotation(transform.rotation * Quat::from_rotation_x(FRAC_PI_2)),
                    Visibility::default(),
                    Collider::cylinder(0.1, 0.1),
                    RigidBody::Dynamic,
                    Velocity {
                        linvel: transform.rotation * Vec3::Z * cannon.shell_speed + Vec3::Y * 2.0,
                        ..default()
                    },
                    TankCannonShell::default(),
                    ActiveEvents::COLLISION_EVENTS,
                ))
                .with_child((
                    Transform::from_scale(Vec3::splat(0.025)),
                    SceneRoot(game_assets.shell.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        ..Default::default()
                    })),
                ));

            state.cooldown = Timer::from_seconds(cannon.fire_rate_secs, TimerMode::Once);
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct TankCannonShell {
    time_to_live: f32,
    damage: f32,
}

impl Default for TankCannonShell {
    fn default() -> Self {
        Self {
            time_to_live: 1.0,
            damage: 10.0,
        }
    }
}

#[derive(Component, Clone, Debug)]
struct TankCannonShellState {
    time_to_live: Timer,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            TankCameraPlugin,
            TankControllerPlugin,
            TankCannonPlugin,
            GridMaterialPlugin,
            #[cfg(feature = "debug")]
            DebugPlugin,
        ))
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .configure_sets(Update, TankCannonSet.run_if(in_state(GameStates::Playing)))
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(
            PreUpdate,
            (update_camera_input, update_tank_input, update_cannon_input)
                .run_if(in_state(GameStates::Playing)),
        )
        .add_systems(
            Update,
            handle_collision_events.run_if(in_state(GameStates::Playing)),
        )
        .run();
}

#[derive(Component, Clone, Copy, Debug)]
struct CollisionWith {
    pub entity: Entity,
}

/* A system that displays the events. */
fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(other, entity, _) => {
                commands.entity(*entity).insert(CollisionWith { entity: *other });
            },
            _ => {},
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridBindlessMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // Ground
    let size = 32;
    let mesh = Plane3d::default()
        .mesh()
        .size(size as f32, size as f32)
        .build();
    let material = GridBindlessMaterial::new(
        UVec2::new(size, size),
        game_assets.prototype_textures.clone(),
    );
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(Vec3::ZERO),
        Collider::cuboid(size as f32 / 2.0, EPSILON, size as f32 / 2.0),
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        TankCamera { ..default() },
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // tank
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.5, 0.0),
            Visibility::default(),
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
            TankController::default(),
            TankCameraTarget::default(),
            TankCannon::default(),
        ))
        .with_child((
            Transform::from_scale(Vec3::splat(2.0)),
            SceneRoot(game_assets.tank.clone()),
        ));
}

fn update_camera_input(
    mut input: ResMut<TankCameraInput>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let mouse_delta = mouse_motion.read().map(|event| event.delta).sum::<Vec2>();
    let scroll_delta = scroll_events
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.005,
        })
        .sum::<f32>();

    input.orbit = if mouse_input.pressed(MouseButton::Right) {
        mouse_delta
    } else {
        Vec2::ZERO
    };

    input.zoom = scroll_delta;
}

fn update_tank_input(mut input: ResMut<TankControllerInput>, keyboard: Res<ButtonInput<KeyCode>>) {
    input.forward = if keyboard.pressed(KeyCode::KeyW) {
        1.0
    } else if keyboard.pressed(KeyCode::KeyS) {
        -1.0
    } else {
        0.0
    };

    input.steer = if keyboard.pressed(KeyCode::KeyA) {
        1.0
    } else if keyboard.pressed(KeyCode::KeyD) {
        -1.0
    } else {
        0.0
    };
}

fn update_cannon_input(mut input: ResMut<TankCannonInput>, keyboard: Res<ButtonInput<KeyCode>>) {
    input.fire = keyboard.pressed(KeyCode::Space);
}
