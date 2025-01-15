use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy_replicon_renet2::renet2::{ConnectionConfig, RenetClient};
use renet2_netcode::{
    ClientAuthentication, ClientSocket, NativeSocket, NetcodeClientTransport,
};

pub async fn create_client(
    address: String,
    config: ConnectionConfig,
    protocol_id: u64,
) -> Result<(RenetClient, NetcodeClientTransport), String> {
    let http_path = format!("http://{}:5000/native", address);
    let server_port = ureq::get(&http_path)
        .call()
        .unwrap()
        .into_string()
        .unwrap()
        .parse::<u16>()
        .unwrap();

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
        protocol_id,
    };

    let client = RenetClient::new(config, client_socket.is_reliable());
    let transport =
        NetcodeClientTransport::new(current_time, authentication, client_socket).unwrap();

    Ok((client, transport))
}
