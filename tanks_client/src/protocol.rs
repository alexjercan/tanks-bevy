use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    renet2::{ConnectionConfig, RenetClient},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

use network::prelude::*;

#[cfg(not(target_family = "wasm"))]
use native::create_client;

#[cfg(target_family = "wasm")]
use wasm::create_client;

pub mod prelude {
    pub use super::{
        ClientConnectEvent, ClientProtocolPlugin, ClientProtocolSet, LocalPlayer, LocalPlayerEntity,
    };
}

/// The ClientConnectEvent is an event that is sent when the client wants to connect to a server
/// with the given address.
#[derive(Debug, Clone, Event)]
pub struct ClientConnectEvent {
    pub address: String,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct LocalPlayer(pub ClientId);

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct LocalPlayerEntity(pub Entity);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientProtocolSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProtocolPlugin;

impl Plugin for ClientProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkPlugin);
        app.add_plugins(RepliconRenetPlugins);

        app.add_event::<ClientConnectEvent>();

        app.add_systems(
            Update,
            (handle_client_connect)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<RenetClient>)),
        );
        app.add_systems(
            Update,
            (update_local_player_entity)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<LocalPlayerEntity>))
                .run_if(resource_exists::<LocalPlayer>),
        );
    }
}

pub fn handle_client_connect(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut connect_events: EventReader<ClientConnectEvent>,
) {
    for ClientConnectEvent { address } in connect_events.read() {
        let config = ConnectionConfig::from_channels(
            channels.get_server_configs(),
            channels.get_client_configs(),
        );

        let (client, transport) = create_client(address.clone(), config);

        commands.insert_resource(LocalPlayer(ClientId::new(transport.client_id())));
        commands.insert_resource(client);
        commands.insert_resource(transport);
    }
}

fn update_local_player_entity(
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    q_player: Query<(Entity, &Player)>,
) {
    for (entity, player) in q_player.iter() {
        if player.client_id == **local_player {
            commands.insert_resource(LocalPlayerEntity(entity));
        }
    }
}

#[cfg(not(target_family = "wasm"))]
mod native {
    use std::{
        net::{Ipv4Addr, SocketAddr, UdpSocket},
        time::SystemTime,
    };

    use bevy_replicon_renet2::renet2::{ConnectionConfig, RenetClient};
    use renet2_netcode::{
        ClientAuthentication, ClientSocket, NativeSocket, NetcodeClientTransport,
    };

    use super::PROTOCOL_ID;

    pub fn create_client(
        address: String,
        config: ConnectionConfig,
    ) -> (RenetClient, NetcodeClientTransport) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let http_path = format!("http://{}:5000/native", address);
        let server_port = runtime.block_on(async move {
            reqwest::get(http_path)
                .await
                .unwrap()
                .json::<u16>()
                .await
                .unwrap()
        });
        let server_addr = SocketAddr::new(address.parse().unwrap(), server_port);

        let client_socket = NativeSocket::new(
            UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)).unwrap(),
        )
        .unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        let authentication = ClientAuthentication::Unsecure {
            socket_id: 0,
            server_addr,
            client_id,
            user_data: None,
            protocol_id: PROTOCOL_ID,
        };

        let client = RenetClient::new(config, client_socket.is_reliable());
        let transport =
            NetcodeClientTransport::new(current_time, authentication, client_socket).unwrap();

        (client, transport)
    }
}

#[cfg(target_family = "wasm")]
mod wasm {
    pub fn create_client(
        address: String,
        config: ConnectionConfig,
    ) -> (RenetClient, NetcodeClientTransport) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let http_path = format!("http://{}:5000/wasm", address);
        let (wt_port, cert_hash, ws_port) = runtime.block_on(async move {
            reqwest::get(http_path)
                .await
                .unwrap()
                .json::<(u16, ServerCertHash, u16)>()
                .await
                .unwrap()
        });

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        if webtransport_is_available_with_cert_hashes() {
            let server_addr = SocketAddr::new(address.parse().unwrap(), wt_port);
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: 0,
                socket_id: 1,
                server_addr,
                user_data: None,
            };
            let socket_config = WebTransportClientConfig {
                server_dest: server_addr.into(),
                congestion_control: CongestionControl::default(),
                server_cert_hashes: Vec::from([cert_hash]),
            };
            let socket = WebTransportClient::new(socket_config);

            let client = RenetClient::new(connection_config, socket.is_reliable());
            let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

            (client, transport)
        } else {
            let server_url = format!("ws://{}:{}/ws", address, ws_port);
            let socket_config = WebSocketClientConfig { server_url };
            let server_addr = socket_config.server_address().unwrap();
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: 0,
                socket_id: 2,
                server_addr,
                user_data: None,
            };

            let socket = WebSocketClient::new(socket_config).unwrap();
            let client = RenetClient::new(connection_config, socket.is_reliable());
            let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

            (client, transport)
        }
    }
}

