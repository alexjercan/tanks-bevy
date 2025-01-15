use std::net::SocketAddr;

use bevy_replicon_renet2::renet2::{ConnectionConfig, RenetClient};
use renet2_netcode::{
    webtransport_is_available_with_cert_hashes, ClientAuthentication, ClientSocket,
    CongestionControl, NetcodeClientTransport, ServerCertHash, WebSocketClient,
    WebSocketClientConfig, WebTransportClient, WebTransportClientConfig,
};
use wasm_timer::{SystemTime, UNIX_EPOCH};

async fn get_server_details(http_path: String) -> (u16, ServerCertHash, u16) {
    reqwest::get(http_path)
        .await
        .unwrap()
        .json::<(u16, ServerCertHash, u16)>()
        .await
        .unwrap()
}

pub fn create_client(
    address: String,
    config: ConnectionConfig,
    protocol_id: u64,
) -> (RenetClient, NetcodeClientTransport) {
    let http_path = format!("http://{}:5000/wasm", address);

    // TODO: how to run async code in wasm_bindgen?
    // let (wt_port, cert_hash, ws_port) = block_on(get_server_details(http_path));

    let wt_port = 54369;
    let cert_hash = ServerCertHash { hash: [153,146,19,35,202,50,61,87,66,235,178,6,92,122,9,196,140,58,49,87,150,159,109,146,11,230,44,211,90,249,139,60] };
    let ws_port = 46713;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    if webtransport_is_available_with_cert_hashes() == false {
        let server_addr = SocketAddr::new(address.parse().unwrap(), wt_port);
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id,
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

        let client = RenetClient::new(config, socket.is_reliable());
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        (client, transport)
    } else {
        let server_url = url::Url::parse(&format!("ws://{}:{}/ws", address, ws_port)).unwrap();
        let socket_config = WebSocketClientConfig { server_url };
        let server_addr = socket_config.server_address().unwrap();
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id,
            socket_id: 2,
            server_addr,
            user_data: None,
        };

        let socket = WebSocketClient::new(socket_config).unwrap();
        let client = RenetClient::new(config, socket.is_reliable());
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        (client, transport)
    }
}
