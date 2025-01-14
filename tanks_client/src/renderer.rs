use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::{GameAssets, RendererPlugin};
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    pub tank: Handle<Scene>,
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

#[derive(Component, Clone, Copy, Debug)]
struct ClientRenderer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::MainMenu)
                .load_collection::<GameAssets>(),
        );

        app.add_systems(OnEnter(GameStates::AssetLoading), spawn_loading_ui);
        app.add_systems(OnEnter(GameStates::Playing), spawn_renderer);
        app.add_systems(
            Update,
            (
                add_ground_cosmetics,
                add_player_cosmetics,
                add_shell_cosmetics,
            )
                .run_if(in_state(GameStates::Playing)),
        );
    }
}

fn spawn_loading_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(GameStates::AssetLoading),
    ));
}

fn spawn_renderer(mut commands: Commands) {
    commands.spawn((
        Name::new("DirectionalLight"),
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameStates::Playing),
    ));
}

fn add_ground_cosmetics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_ground: Query<(Entity, &Ground), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
) {
    for (entity, Ground { width, height }) in q_ground.iter() {
        let mesh = Plane3d::default().mesh().size(*width, *height).build();
        let material = StandardMaterial {
            base_color_texture: Some(game_assets.prototype_textures[0].clone_weak()),
            ..Default::default()
        };
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
            ClientRenderer,
        ));
    }
}

fn add_player_cosmetics(
    mut commands: Commands,
    q_player: Query<(Entity, &Player), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, Player { name, color, .. }) in q_player.iter() {
        // TODO: add cosmetics for player
        info!("Adding cosmetics for player: {}", name);
        let material = StandardMaterial {
            base_color: *color,
            ..Default::default()
        };
        commands
            .entity(entity)
            .insert((Visibility::default(), ClientRenderer))
            .with_child((
                Transform::from_scale(Vec3::splat(2.0)),
                SceneRoot(game_assets.tank.clone()),
                MeshMaterial3d(materials.add(material)),
            ));
    }
}

fn add_shell_cosmetics(
    mut commands: Commands,
    q_shell: Query<(Entity, &Shell), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
) {
    for (entity, _) in q_shell.iter() {
        commands
            .entity(entity)
            .insert((Visibility::default(), ClientRenderer))
            .with_child((
                Transform::from_scale(Vec3::splat(0.025)),
                SceneRoot(game_assets.shell.clone()),
            ));
    }
}
