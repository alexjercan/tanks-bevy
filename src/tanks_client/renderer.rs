use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use blenvy::*;

pub mod prelude {
    pub use super::RendererPlugin;
}

#[derive(Component, Clone, Copy, Debug)]
struct ClientRenderer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlenvyPlugin {
            export_registry: false,
            ..default()
        });

        app.add_systems(OnEnter(GameStates::Playing), spawn_renderer);
        app.add_systems(
            Update,
            (add_player_cosmetics, add_shell_cosmetics).run_if(in_state(GameStates::Playing)),
        );
    }
}

fn spawn_renderer(mut commands: Commands) {
    commands.spawn((
        Name::new("DirectionalLight"),
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(GameStates::Playing),
    ));

    // here we actually spawn our game world/level
    commands.spawn((
        BlueprintInfo::from_path("levels/World.glb"), // all we need is a Blueprint info...
        SpawnBlueprint, // and spawnblueprint to tell blenvy to spawn the blueprint now
        HideUntilReady, // only reveal the level once it is ready
        GameWorldTag,
    ));
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
