use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, SystemTime},
};

use bevy::{app::ScheduleRunnerPlugin, prelude::*, winit::WinitPlugin};
use bevy_rapier3d::prelude::*;
use bevy_renet::{netcode::*, renet::*, RenetServerPlugin};
use tanks::prelude::*;

#[derive(Resource, Default, Debug)]
struct Lobby {
    names: HashMap<ClientId, String>,
    players: HashMap<ClientId, Entity>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
    app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 60.0,
    )));

    // Network
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);

    let (server, transport) = new_renet_server();
    app.insert_resource(server);
    app.insert_resource(transport);

    app.init_resource::<Lobby>();

    app.add_systems(
        Update,
        (
            handle_server_events,
            handle_client_messages,
            sync_tank_transforms,
        )
            .run_if(resource_exists::<RenetServer>),
    );

    // Server Side
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugins(TankControllerPlugin);

    app.add_systems(Startup, setup_game);

    app.run();
}

fn setup_game(mut commands: Commands) {
    // Ground
    let size = 32;
    commands.spawn((
        Transform::from_translation(Vec3::ZERO),
        Collider::cuboid(size as f32 / 2.0, f32::EPSILON, size as f32 / 2.0),
    ));
}

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "0.0.0.0:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

fn handle_server_events(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player {} connected.", client_id);

                let message = ServerMessage::ClientConnectedAck { id: *client_id };
                server.send_message(*client_id, ServerChannel::Message, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {} disconnected: {}", client_id, reason);

                lobby.names.remove(client_id);
                if let Some(id) = lobby.players.remove(client_id) {
                    let message = ServerMessage::DespawnEntity { id };
                    server.broadcast_message_except(*client_id, ServerChannel::Message, message);

                    commands.entity(id).despawn_recursive();
                }

                let message = ServerMessage::ClientLeft { id: *client_id };
                server.broadcast_message_except(*client_id, ServerChannel::Message, message);
            }
        }
    }
}

fn handle_client_messages(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,
    q_entities: Query<(Entity, &NetworkEntity, &Transform)>,
    mut q_controllers: Query<&mut TankControllerInput, With<NetworkEntity>>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Message) {
            let message: ClientMessage = message.into();
            match message {
                ClientMessage::ClientJoin { name } => {
                    info!("Client {} with name {} joined.", client_id, name);

                    lobby.names.insert(client_id, name.clone());

                    let message = ServerMessage::ClientJoined {
                        id: client_id,
                        name,
                    };
                    server.broadcast_message_except(client_id, ServerChannel::Message, message);
                    let message = ServerMessage::ClientJoinAck;
                    server.send_message(client_id, ServerChannel::Message, message);
                }
                ClientMessage::RequestLobbyInfo => {
                    info!("Client {} requested lobby info.", client_id);

                    let message = ServerMessage::LobbyInfo {
                        names: lobby.names.clone(),
                    };
                    server.send_message(client_id, ServerChannel::Message, message);
                }
                ClientMessage::ClientReady => {
                    info!("Client {} is ready.", client_id);

                    for (entity, network_entity, transform) in q_entities.iter() {
                        let message = ServerMessage::SpawnEntity {
                            id: entity,
                            position: transform.translation,
                            rotation: transform.rotation,
                            kind: network_entity.kind,
                        };
                        server.send_message(client_id, ServerChannel::Message, message);
                    }

                    let position = Vec3::new(
                        rand::random::<f32>() * 20. - 10.,
                        0.5,
                        rand::random::<f32>() * 20. - 10.,
                    );
                    let rotation = Quat::IDENTITY;
                    let id = commands
                        .spawn((
                            Transform::from_translation(position).with_rotation(rotation),
                            NetworkEntity {
                                kind: EntityKind::Tank(client_id),
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

                    lobby.players.insert(client_id, id);
                    let message = ServerMessage::SpawnEntity {
                        id,
                        position,
                        rotation,
                        kind: EntityKind::Tank(client_id),
                    };
                    server.broadcast_message(ServerChannel::Message, message);
                }
                ClientMessage::ControllerInput { forward, steer } => {
                    if let Some(id) = lobby.players.get(&client_id) {
                        if let Ok(mut input) = q_controllers.get_mut(*id) {
                            input.forward = forward;
                            input.steer = steer;
                        }
                    }
                }
            }
        }
    }
}

fn sync_tank_transforms(
    mut server: ResMut<RenetServer>,
    q_entities: Query<(Entity, &Transform), With<NetworkEntity>>,
) {
    for (entity, transform) in q_entities.iter() {
        let message = ServerMessage::SyncTransform {
            id: entity,
            position: transform.translation,
            rotation: transform.rotation,
        };
        server.broadcast_message(ServerChannel::Message, message);
    }
}
