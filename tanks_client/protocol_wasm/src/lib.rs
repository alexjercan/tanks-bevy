use std::net::SocketAddr;

use bevy_replicon_renet2::renet2::{ConnectionConfig, RenetClient};
use renet2_netcode::{
    webtransport_is_available_with_cert_hashes, ClientAuthentication, ClientSocket,
    CongestionControl, NetcodeClientTransport, ServerCertHash, WebSocketClient,
    WebSocketClientConfig, WebTransportClient, WebTransportClientConfig,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_timer::SystemTime;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = fetch)]
    fn fetch_with_str(url: &str) -> js_sys::Promise;
}

pub async fn create_client(
    address: String,
    config: ConnectionConfig,
    protocol_id: u64,
) -> Result<(RenetClient, NetcodeClientTransport), String> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = format!("http://{}:5000/wasm", address);

    tracing::info!("getting server info from {}", url);
    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    let resp: Response = resp_value.dyn_into().unwrap();
    let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

    let (wt_port, cert_hash, ws_port) =
        serde_wasm_bindgen::from_value::<(u16, ServerCertHash, u16)>(json).unwrap();
    tracing::debug!("wt_port = {}, cert_hash = {:?}, ws_port = {}", wt_port, cert_hash, ws_port);

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    if webtransport_is_available_with_cert_hashes() {
        let server_addr = SocketAddr::new(address.parse().unwrap(), wt_port);
        tracing::info!("setting up webtransport client (server = {:?})", server_addr);

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

        Ok((client, transport))
    } else {
        tracing::warn!("webtransport with cert hashes is not supported on this platform, falling back to websockets");
        let server_url = url::Url::parse(&format!("ws://{}:{}/ws", address, ws_port)).unwrap();
        tracing::info!("setting up websocket client (server = {:?})", server_url.as_str());

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

        Ok((client, transport))
    }
}
