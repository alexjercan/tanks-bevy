use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use tanks::prelude::*;

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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    MainMenu,
    Playing,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(GridMaterialPlugin);
    app.add_plugins(TankCameraPlugin);

    // #[cfg(feature = "debug")]
    // app.add_plugins(DebugPlugin);

    // Load Assets
    app.init_state::<GameStates>();
    app.enable_state_scoped_entities::<GameStates>();
    app.add_loading_state(
        LoadingState::new(GameStates::AssetLoading)
            .continue_to_state(GameStates::MainMenu)
            .load_collection::<GameAssets>(),
    );

    // Main Menu
    app.add_plugins(MainMenuPlugin);
    app.configure_sets(Update, MainMenuSet.run_if(in_state(GameStates::MainMenu)));
    app.add_systems(OnEnter(GameStates::MainMenu), setup_main_menu);
    app.add_systems(
        Update,
        handle_play_button_pressed.run_if(in_state(GameStates::MainMenu)),
    );

    // Network
    app.add_plugins(ClientPlugin);
    app.configure_sets(Update, ClientSet.run_if(in_state(GameStates::Playing)));

    // Client Side
    app.add_systems(OnEnter(GameStates::Playing), setup_game);
    app.add_systems(
        PreUpdate,
        (update_camera_input, update_tank_input).run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        Update,
        (create_network_entity_graphics).run_if(in_state(GameStates::Playing)),
    );

    app.run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(GameStates::MainMenu)));
    commands.spawn((MainMenuRoot, StateScoped(GameStates::MainMenu)));
}

fn handle_play_button_pressed(
    mut events: EventReader<PlayButtonPressed>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut client_info: ResMut<ClientInfo>,
) {
    for event in events.read() {
        client_info.address = event.address.clone();
        client_info.name = event.name.clone();

        next_state.set(GameStates::Playing);
    }
}

fn setup_game(
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

fn create_network_entity_graphics(
    mut commands: Commands,
    q_entities: Query<(Entity, &NetworkEntity), Added<NetworkEntity>>,
    game_assets: Res<GameAssets>,
    local_player: Res<LocalPlayer>,
) {
    for (entity, network_entity) in q_entities.iter() {
        match network_entity.kind {
            EntityKind::Tank(client_id) => {
                commands
                    .entity(entity)
                    .with_child((
                        Transform::from_scale(Vec3::splat(2.0)),
                        SceneRoot(game_assets.tank.clone()),
                    ));

                if let Some(local_id) = local_player.id {
                    if client_id == local_id {
                        commands.entity(entity).insert(TankCameraTarget::default());
                    }
                }
            }
        }
    }
}
