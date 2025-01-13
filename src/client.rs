use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_renet::client_just_connected;
use leafwing_input_manager::prelude::*;
use tanks::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    MainMenu,
    Connecting,
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
    app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
    app.add_plugins(InputManagerPlugin::<PlayerInputAction>::default());

    app.add_plugins(TankCameraPlugin);

    app.init_state::<GameStates>();
    app.enable_state_scoped_entities::<GameStates>();

    // Load Assets
    app.add_loading_state(
        LoadingState::new(GameStates::AssetLoading)
            .continue_to_state(GameStates::MainMenu)
            .load_collection::<GameAssets>(),
    );

    // Main Menu
    app.add_plugins(MainMenuPlugin);
    app.configure_sets(Update, MainMenuSet.run_if(in_state(GameStates::MainMenu)));

    // Network
    app.add_plugins(ClientPlugin);

    // Renderer
    app.add_plugins(RendererPlugin);
    app.configure_sets(
        Update,
        RendererSet
            .run_if(in_state(GameStates::Playing))
            .run_if(client_connected),
    );

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    app.add_systems(OnEnter(GameStates::MainMenu), setup_main_menu);
    app.add_systems(
        Update,
        handle_play_button_pressed.run_if(in_state(GameStates::MainMenu)),
    );
    app.add_systems(
        Update,
        handle_connecting_done
            .run_if(in_state(GameStates::Connecting))
            .run_if(client_just_connected),
    );
    app.add_systems(OnEnter(GameStates::Playing), setup_game);
    app.add_systems(
        PreUpdate,
        (update_camera_input, update_player_input)
            .run_if(in_state(GameStates::Playing))
            .run_if(client_connected),
    );
    app.add_systems(
        Update,
        (update_camera_target)
            .run_if(in_state(GameStates::Playing))
            .run_if(client_connected),
    );

    app.run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(GameStates::MainMenu),
    ));
    commands.spawn((
        Name::new("MainMenu"),
        MainMenuRoot,
        StateScoped(GameStates::MainMenu),
    ));
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

        next_state.set(GameStates::Connecting);

        client_events.send(ClientConnectEvent {
            address: client_info.address.clone(),
        });
    }
}

fn handle_connecting_done(mut next_state: ResMut<NextState<GameStates>>) {
    next_state.set(GameStates::Playing);
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(Axis)]
    Zoom,
}

fn update_camera_input(mut q_camera: Query<(&mut TankCameraInput, &ActionState<CameraMovement>)>) {
    for (mut input, action) in q_camera.iter_mut() {
        input.zoom = action.value(&CameraMovement::Zoom);
        input.orbit = action.axis_pair(&CameraMovement::Pan);
    }
}

fn update_camera_target(
    q_player: Query<(&Player, &Transform)>,
    local_player: Res<LocalPlayer>,
    mut q_camera: Query<&mut TankCameraTransformTarget>,
) {
    for (Player { client_id, .. }, transform) in q_player.iter() {
        if *client_id == **local_player {
            for mut target in q_camera.iter_mut() {
                target.focus = transform.translation;
            }
        }
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum PlayerInputAction {
    #[actionlike(DualAxis)]
    Move,
}

#[derive(Component, Clone, Debug, Copy, Deref, DerefMut, Default)]
struct PlayerInputMove(Vec2);

fn update_player_input(
    mut input: EventWriter<PlayerInputEvent>,
    mut q_input: Query<(&mut PlayerInputMove, &ActionState<PlayerInputAction>)>,
) {
    for (mut prev, action) in q_input.iter_mut() {
        let movement = action.clamped_axis_pair(&PlayerInputAction::Move);

        if movement.x != prev.x || movement.y != prev.y {
            **prev = movement;
            input.send(PlayerInputEvent(movement));
        }
    }
}

fn setup_game(
    mut commands: Commands,
    client_info: Res<ClientInfo>,
    mut join: EventWriter<PlayerJoinEvent>,
) {
    commands.spawn((
        Name::new("DirectionalLight"),
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameStates::Playing),
    ));

    let input_map = InputMap::default()
        .with_dual_axis(CameraMovement::Pan, MouseMove::default())
        .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y);

    commands.spawn((
        Name::new("Camera3d"),
        TankCameraInput::default(),
        TankCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        InputManagerBundle::with_map(input_map),
        StateScoped(GameStates::Playing),
    ));

    let input_map =
        InputMap::default().with_dual_axis(PlayerInputAction::Move, VirtualDPad::wasd());

    commands.spawn((
        Name::new("PlayerInput"),
        PlayerInputMove::default(),
        InputManagerBundle::with_map(input_map),
    ));

    join.send(PlayerJoinEvent {
        name: client_info.name.clone(),
        color: Color::srgb(0.0, 0.0, 1.0),
    });
}
