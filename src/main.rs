use std::f32::EPSILON;

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
pub struct TankController {
    /// The movement speed of the tank
    pub move_speed: f32,
    /// The rotation speed of the tank
    pub rotation_speed: f32,
}

impl Default for TankController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            rotation_speed: 2.0,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct TankControllerInput {
    pub forward: f32,
    pub steer: f32,
}

impl TankControllerInput {
    fn has_input(&self) -> bool {
        self.forward != 0.0 || self.steer != 0.0
    }
}

pub struct TankControllerPlugin;

impl Plugin for TankControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TankControllerInput>()
            .add_systems(FixedUpdate, player_movement);
    }
}

fn player_movement(
    time: Res<Time>,
    input: Res<TankControllerInput>,
    mut player: Query<(
        &TankController,
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut y_rotation: Local<f32>,
) {
    if !input.has_input() {
        return;
    }

    let Ok((tank, mut transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };

    let delta_time = time.delta_secs();
    let mut movement = Vec3::new(0.0, 0.0, input.forward) * tank.move_speed;

    if output.map(|o| o.grounded).unwrap_or(false) {
        *vertical_movement = 0.0;
    }

    if input.steer != 0.0 {
        *y_rotation += input.steer * tank.rotation_speed * delta_time;
    }

    movement.y = *vertical_movement;
    *vertical_movement += -9.81 * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));

    transform.rotation = Quat::from_rotation_y(*y_rotation);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            TankCameraPlugin,
            TankControllerPlugin,
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
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(
            PreUpdate,
            (update_camera_input, update_tank_input).run_if(in_state(GameStates::Playing)),
        )
        .run();
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
