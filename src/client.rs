use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_renet::{client_connected, netcode::*, renet::*, RenetClientPlugin};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};
use tanks::prelude::*;

#[derive(Resource, Default, Debug)]
struct Lobby {
    /// The names of the clients in the lobby.
    names: HashMap<ClientId, String>,
    /// Map from server entity id to client entity id.
    entities: HashMap<Entity, Entity>,
}

#[derive(Resource, Default, Debug)]
pub struct ControllerInput {
    pub forward: f32,
    pub steer: f32,
}

#[derive(Resource, Default, Debug)]
pub struct LocalPlayer {
    pub id: Option<ClientId>,
}

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
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.init_resource::<ClientInfo>();
    app.init_resource::<LocalPlayer>();
    app.init_resource::<Lobby>();

    app.add_systems(
        Update,
        setup_network
            .run_if(in_state(GameStates::Playing))
            .run_if(not(resource_exists::<RenetClient>)),
    );
    app.add_systems(
        Update,
        (handle_server_messages, sync_tank_input)
            .run_if(in_state(GameStates::Playing))
            .run_if(client_connected),
    );

    // Input
    app.init_resource::<ControllerInput>();

    // Client Side
    app.add_systems(OnEnter(GameStates::Playing), setup_game);
    app.add_systems(
        PreUpdate,
        (update_camera_input, update_tank_input).run_if(in_state(GameStates::Playing)),
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

fn update_tank_input(mut input: ResMut<ControllerInput>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut forward = 0.0;

    if keyboard.pressed(KeyCode::KeyW) {
        forward += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        forward -= 1.0;
    }

    let mut steer = 0.0;

    if keyboard.pressed(KeyCode::KeyA) {
        steer += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        steer -= 1.0;
    }

    if input.forward != forward || input.steer != steer {
        input.forward = forward;
        input.steer = steer;
    }
}

fn sync_tank_input(
    mut client: ResMut<RenetClient>,
    input: Res<ControllerInput>,
) {
    if input.is_changed() {
        let message = ClientMessage::ControllerInput {
            forward: input.forward,
            steer: input.steer,
        };
        client.send_message(ClientChannel::Message, message);
    }
}

fn setup_network(mut commands: Commands, client_info: Res<ClientInfo>) {
    let server_addr = client_info.address.parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    commands.insert_resource(client);
    commands.insert_resource(transport);
}

fn handle_server_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut local_player: ResMut<LocalPlayer>,
    mut lobby: ResMut<Lobby>,
    client_info: Res<ClientInfo>,
    game_assets: Res<GameAssets>,
    mut q_transform: Query<&mut Transform, With<NetworkEntity>>,
) {
    while let Some(message) = client.receive_message(ServerChannel::Message) {
        let message: ServerMessage = message.into();
        match message {
            ServerMessage::ClientConnectedAck { id } => {
                info!("Connected to server with id: {}", id);
                local_player.id = Some(id);

                let message = ClientMessage::ClientJoin {
                    name: client_info.name.clone(),
                };
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::ClientJoined { id, name } => {
                info!("Player {} joined with name: {}", id, name);

                lobby.names.insert(id, name.clone());
            }
            ServerMessage::ClientJoinAck => {
                info!("Successfully joined the lobby.");

                let message = ClientMessage::RequestLobbyInfo;
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::LobbyInfo { names } => {
                info!("Received lobby info: {:?}", names);

                lobby.names = names;

                let message = ClientMessage::ClientReady;
                client.send_message(ClientChannel::Message, message);
            }
            ServerMessage::ClientLeft { id } => {
                info!("Player {} left the lobby.", id);

                lobby.names.remove(&id);
            }
            ServerMessage::SpawnEntity {
                id,
                position,
                rotation,
                kind,
            } => {
                info!(
                    "Spawning entity {} ({:?}) at {:?} with rotation {:?}.",
                    id, kind, position, rotation
                );

                match kind {
                    EntityKind::Tank(client_id) => {
                        let entity = commands
                            .spawn((
                                Transform::from_translation(position).with_rotation(rotation),
                                Visibility::default(),
                                NetworkEntity { kind },
                            ))
                            .with_child((
                                Transform::from_scale(Vec3::splat(2.0)),
                                SceneRoot(game_assets.tank.clone()),
                            ))
                            .id();

                        if let Some(local_id) = local_player.id {
                            if client_id == local_id {
                                commands.entity(entity).insert(TankCameraTarget::default());
                            }
                        }

                        lobby.entities.insert(id, entity);
                    }
                }
            }
            ServerMessage::DespawnEntity { id } => {
                info!("Despawning entity {}.", id);

                if let Some(local_id) = lobby.entities.remove(&id) {
                    commands.entity(local_id).despawn_recursive();
                }
            }
            ServerMessage::SyncTransform {
                id,
                position,
                rotation,
            } => {
                info!(
                    "Syncing transform of entity {} to {:?} with rotation {:?}.",
                    id, position, rotation
                );

                if let Some(entity) = lobby.entities.get(&id) {
                    if let Ok(mut transform) = q_transform.get_mut(*entity) {
                        transform.translation = position;
                        transform.rotation = rotation;
                    }
                }
            }
        }
    }
}
