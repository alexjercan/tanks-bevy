use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};
use warp::Filter;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    netcode::{
        BoxedSocket, NativeSocket, NetcodeServerTransport, ServerAuthentication, ServerCertHash,
        ServerSetupConfig, ServerSocket, WebSocketServer, WebSocketServerConfig,
        WebTransportServer, WebTransportServerConfig,
    },
    renet2::{ConnectionConfig, RenetServer},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

use network::prelude::*;

pub mod prelude {
    pub use super::{
        ClientConnectedEvent, ClientDisconnectedEvent, ServerProtocolPlugin, ServerProtocolSet,
    };
}

/// The ClientConnectedEvent is an event that is sent when a client connects to the server
#[derive(Debug, Clone, Event)]
pub struct ClientConnectedEvent {
    pub client_id: ClientId,
}

/// The ClientDisconnectedEvent is an event that is sent when a client disconnects from the server
#[derive(Debug, Clone, Event)]
pub struct ClientDisconnectedEvent {
    pub client_id: ClientId,
    pub reason: String,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerProtocolSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProtocolPlugin;

impl Plugin for ServerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkPlugin);
        app.add_plugins(RepliconRenetPlugins);

        app.add_event::<ClientConnectedEvent>();
        app.add_event::<ClientDisconnectedEvent>();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        app.insert_resource(TokioRuntime(runtime));

        app.add_systems(Startup, start_server);

        app.add_systems(
            Update,
            (handle_server_events)
                .in_set(ServerProtocolSet)
                .run_if(resource_exists::<RenetServer>),
        );
    }
}

#[derive(Resource, Debug, Deref, DerefMut)]
struct TokioRuntime(tokio::runtime::Runtime);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientConnectionInfo {
    native_port: u16,
    wt_port: u16,
    ws_port: u16,
    cert_hash: ServerCertHash,
}

fn start_server(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    runtime: Res<TokioRuntime>,
) {
    let server = RenetServer::new(ConnectionConfig::from_channels(
        channels.get_server_configs(),
        channels.get_client_configs(),
    ));

    let max_clients = 64;

    // HTTP server
    let http_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5000);

    // Native socket
    let native_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5001);
    let native_socket = NativeSocket::new(UdpSocket::bind(native_addr).unwrap()).unwrap();

    // WebTransport socket
    let wt_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5002);
    let (wt_socket, cert_hash) = {
        let (config, cert_hash) = WebTransportServerConfig::new_selfsigned(wt_addr, max_clients);
        (
            WebTransportServer::new(config, runtime.handle().clone()).unwrap(),
            cert_hash,
        )
    };

    // WebSocket socket
    let ws_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5003);
    let ws_socket = {
        let config = WebSocketServerConfig::new(ws_addr, max_clients);
        WebSocketServer::new(config, runtime.handle().clone()).unwrap()
    };

    let client_connection_info = ClientConnectionInfo {
        native_port: native_socket.addr().unwrap().port(),
        wt_port: wt_socket.addr().unwrap().port(),
        ws_port: ws_socket.url().port().unwrap(),
        cert_hash,
    };
    debug!("Client connection info: {:?}", client_connection_info);

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerSetupConfig {
        current_time,
        max_clients,
        protocol_id: PROTOCOL_ID,
        socket_addresses: vec![
            vec![native_socket.addr().unwrap()],
            vec![wt_socket.addr().unwrap()],
            vec![ws_socket.addr().unwrap()],
        ],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new_with_sockets(
        server_config,
        Vec::from([
            BoxedSocket::new(native_socket),
            BoxedSocket::new(wt_socket),
            BoxedSocket::new(ws_socket),
        ]),
    )
    .unwrap();

    commands.insert_resource(server);
    commands.insert_resource(transport);

    runtime.spawn(async move { run_http_server(http_addr, client_connection_info).await });
}

async fn run_http_server(http_addr: SocketAddr, client_connection_info: ClientConnectionInfo) {
    let native_port = client_connection_info.native_port;
    let wt_port = client_connection_info.wt_port;
    let ws_port = client_connection_info.ws_port;
    let cert_hash = client_connection_info.cert_hash;

    let native = warp::path!("native").map(move || {
        info!("Native port: {}", native_port);
        warp::reply::json(&native_port)
    });

    let cors = warp::cors().allow_any_origin();
    let wasm = warp::path!("wasm")
        .map(move || {
            info!(
                "WebTransport port: {}, cert hash: {:?}, WebSocket port: {}",
                wt_port, cert_hash, ws_port
            );
            warp::reply::json(&(&wt_port, &cert_hash, &ws_port))
        })
        .with(cors);

    let routes = warp::get().and(native.or(wasm));

    warp::serve(routes).run(http_addr).await;
}

fn handle_server_events(
    mut events: EventReader<ServerEvent>,
    mut connected: EventWriter<ClientConnectedEvent>,
    mut disconnected: EventWriter<ClientDisconnectedEvent>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                debug!("Client {:?} connected.", client_id);

                connected.send(ClientConnectedEvent {
                    client_id: *client_id,
                });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                debug!("Client {:?} disconnected: {}", client_id, reason);

                disconnected.send(ClientDisconnectedEvent {
                    client_id: *client_id,
                    reason: reason.to_string(),
                });
            }
        }
    }
}
