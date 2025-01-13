use bevy::{asset::AssetMetaCheck, input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel}, prelude::*};
use bevy_asset_loader::prelude::*;
use tanks::prelude::*;

#[derive(Resource, Debug)]
pub struct ClientInfo {
    pub address: String,
    pub name: String,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:5000".to_string(),
            name: "Player".to_string(),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ClientRenderer;

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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    MainMenu,
    Playing,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tanks".to_string(),
                    // Bind to canvas included in `index.html`
                    canvas: Some("#bevy".to_owned()),
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5 and Ctrl+R
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    );

    app.add_plugins(GridMaterialPlugin);
    app.add_plugins(TankCameraPlugin);

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
    app.init_resource::<ClientInfo>();

    // Network
    app.add_plugins(ClientPlugin);

    app.add_systems(OnEnter(GameStates::MainMenu), setup_main_menu);
    app.add_systems(
        Update,
        handle_play_button_pressed.run_if(in_state(GameStates::MainMenu)),
    );
    app.add_systems(OnEnter(GameStates::Playing), setup_game);
    app.add_systems(
        PreUpdate,
        (update_camera_input).run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        Update,
        (add_ground_cosmetics, add_player_cosmetics).run_if(in_state(GameStates::Playing)),
    );

    app.run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(GameStates::MainMenu)));
    commands.spawn((MainMenuRoot, StateScoped(GameStates::MainMenu)));
}

fn handle_play_button_pressed(
    mut button_events: EventReader<PlayButtonPressed>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut client_info: ResMut<ClientInfo>,
    mut client_events: EventWriter<ClientConnectEvent>,
) {
    for event in button_events.read() {
        client_info.address = event.address.clone();
        client_info.name = event.name.clone();

        next_state.set(GameStates::Playing);

        client_events.send(ClientConnectEvent {
            address: client_info.address.clone(),
        });
    }
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

fn setup_game(mut commands: Commands) {
    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameStates::Playing),
    ));

    // camera
    commands.spawn((
        TankCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameStates::Playing),
    ));
}

fn add_ground_cosmetics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridBindlessMaterial>>,
    q_ground: Query<(Entity, &Ground), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
) {
    for (entity, Ground { width, height }) in q_ground.iter() {
        let mesh = Plane3d::default().mesh().size(*width, *height).build();
        let material = GridBindlessMaterial::new(
            UVec2::new(*width as u32, *height as u32),
            game_assets.prototype_textures.clone(),
        );
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
        ));
    }
}

fn add_player_cosmetics(
    mut commands: Commands,
    q_player: Query<(Entity, &Player), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
    local_player: Res<LocalPlayer>,
) {
    for (entity, Player { client_id }) in q_player.iter() {
        commands
            .entity(entity)
            .insert((Visibility::default(),))
            .with_child((
                Transform::from_scale(Vec3::splat(2.0)),
                SceneRoot(game_assets.tank.clone()),
            ));

            if *client_id == **local_player {
                commands.entity(entity).insert(TankCameraTarget::default());
            }
    }
}
