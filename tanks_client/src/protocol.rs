use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{
    renet2::{ConnectionConfig, RenetClient},
    RenetChannelsExt, RepliconRenetPlugins,
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use network::prelude::*;

#[cfg(not(target_family = "wasm"))]
use protocol_native::create_client;

#[cfg(target_family = "wasm")]
use protocol_wasm::create_client;

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

#[derive(Resource, Debug)]
struct ConnectTask(pub Task<CommandQueue>);

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
            (generate_connect_task)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<RenetClient>))
                .run_if(not(resource_exists::<ConnectTask>)),
        );
        app.add_systems(
            Update,
            (handle_connect_task)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<RenetClient>))
                .run_if(resource_exists::<ConnectTask>),
        );
        app.add_systems(
            Update,
            (update_local_player_entity)
                .in_set(ClientProtocolSet)
                .run_if(not(resource_exists::<LocalPlayerEntity>))
                .run_if(resource_exists::<LocalPlayer>),
        );

        app.add_systems(
            OnExit(GameStates::Playing),
            (disconnect_client)
                .in_set(ClientProtocolSet)
                .run_if(resource_exists::<LocalPlayer>),
        );
    }
}

pub fn generate_connect_task(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut connect_events: EventReader<ClientConnectEvent>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for ClientConnectEvent { address } in connect_events.read() {
        let address = address.clone();
        let config = ConnectionConfig::from_channels(
            channels.get_server_configs(),
            channels.get_client_configs(),
        );

        let task = thread_pool.spawn(async move {
            let (client, transport) = create_client(address, config, PROTOCOL_ID).await.unwrap();

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                world.insert_resource(LocalPlayer(ClientId::new(transport.client_id())));
                world.insert_resource(client);
                world.insert_resource(transport);
            });

            command_queue
        });

        commands.insert_resource(ConnectTask(task));
    }
}

fn handle_connect_task(mut commands: Commands, mut task: ResMut<ConnectTask>) {
    if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
        commands.append(&mut commands_queue);
        commands.remove_resource::<ConnectTask>();
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

fn disconnect_client(mut commands: Commands, mut client: ResMut<RenetClient>) {
    client.disconnect();
    commands.remove_resource::<LocalPlayer>();
    commands.remove_resource::<LocalPlayerEntity>();
    commands.remove_resource::<RenetClient>();
}
