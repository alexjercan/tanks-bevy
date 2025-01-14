use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::{ConnectionConfig, RenetClient}};
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RenetChannelsExt;
use serde::{Deserialize, Serialize};
use crate::network::prelude::{NetworkPlugin, PROTOCOL_ID};
use bevy_replicon_renet::RepliconRenetPlugins;

pub mod prelude {
    pub use super::{ClientProtocolPlugin, ClientProtocolSet, ClientConnectEvent, LocalPlayer};
}

/// The ClientConnectEvent is an event that is sent when the client wants to connect to a server
/// with the given address.
#[derive(Debug, Clone, Event)]
pub struct ClientConnectEvent {
    pub address: String,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct LocalPlayer(pub ClientId);

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
    }
}

fn handle_client_connect(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut connect_events: EventReader<ClientConnectEvent>,
) {
    for ClientConnectEvent { address } in connect_events.read() {
        let server_channels_config = channels.get_server_configs();
        let client_channels_config = channels.get_client_configs();

        let client = RenetClient::new(ConnectionConfig {
            server_channels_config,
            client_channels_config,
            ..Default::default()
        });

        let server_addr = address.parse().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
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

        commands.insert_resource(LocalPlayer(ClientId::new(client_id)));
        commands.insert_resource(client);
        commands.insert_resource(transport);
    }
}
