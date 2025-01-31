use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_asset_loader::prelude::*;

use crate::prelude::*;
use ::utils::prelude::*;

pub mod prelude {
    pub use super::ClientPlugin;
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
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
        app.add_plugins(ClientProtocolPlugin);
        app.add_plugins(RendererPlugin);
        app.add_plugins(MainMenuPlugin);
        app.add_plugins(TankCameraPlugin);
        app.add_plugins(TankInputPlugin);
        app.add_plugins(GameGuiPlugin);
        app.add_plugins(AudioEffectsPlugin);
        app.add_plugins(DespawnAfterPlugin);

        // FIXME: For now we disable particle effects on wasm because it's not working
        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(ParticleEffectsPlugin);

        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);

        app.init_state::<GameStates>();
        app.enable_state_scoped_entities::<GameStates>();
        app.add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::MainMenu)
                .load_collection::<GameAssets>(),
        );

        app.add_systems(OnEnter(GameStates::AssetLoading), spawn_loading_ui);
        app.add_systems(
            Update,
            handle_play_button_pressed.run_if(in_state(GameStates::MainMenu)),
        );
        app.add_systems(OnEnter(GameStates::Connecting), spawn_connecting_ui);
        app.add_systems(
            Update,
            handle_connecting_done
                .run_if(in_state(GameStates::Connecting))
                .run_if(client_just_connected),
        );
        app.add_systems(OnEnter(GameStates::Playing), (setup_game, hide_cursor));
        app.add_systems(
            Update,
            (handle_player_died, handle_state_scoped).run_if(in_state(GameStates::Playing)),
        );
        app.add_systems(OnExit(GameStates::Playing), show_cursor);
    }
}

pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}

fn spawn_loading_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(GameStates::AssetLoading),
    ));

    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            StateScoped(GameStates::AssetLoading),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(icon),
                Node {
                    // This will set the logo to be 200px wide, and auto adjust its height
                    width: Val::Px(200.0),
                    ..default()
                },
            ));
        });
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

fn spawn_connecting_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(GameStates::Connecting),
    ));

    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            StateScoped(GameStates::Connecting),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(icon),
                Node {
                    // This will set the logo to be 200px wide, and auto adjust its height
                    width: Val::Px(200.0),
                    ..default()
                },
            ));
        });
}

fn handle_connecting_done(mut next_state: ResMut<NextState<GameStates>>) {
    next_state.set(GameStates::Playing);
}

fn setup_game(client_info: Res<ClientInfo>, mut join: EventWriter<PlayerJoinEvent>) {
    join.send(PlayerJoinEvent {
        name: client_info.name.clone(),
        color: Color::srgb(0.0, 0.0, 1.0),
    });
}

fn handle_player_died(
    mut commands: Commands,
    mut died: EventReader<PlayerDiedEvent>,
    local_player: Res<LocalPlayer>,
) {
    for event in died.read() {
        if event.client_id == **local_player {
            commands.remove_resource::<LocalPlayerEntity>();
        }
    }
}

fn handle_state_scoped(
    mut commands: Commands,
    q_entity: Query<Entity, (With<NetworkEntity>, Without<StateScoped<GameStates>>)>,
) {
    for entity in q_entity.iter() {
        commands
            .entity(entity)
            .insert((StateScoped(GameStates::Playing),));
    }
}
