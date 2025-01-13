use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::{GridBindlessMaterial, GridMaterialPlugin, Ground, Player};

pub mod prelude {
    pub use super::{GameAssets, RendererPlugin, RendererSet};
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    pub tank: Handle<Scene>,
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererSet;

#[derive(Component, Clone, Copy, Debug)]
struct ClientRenderer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GridMaterialPlugin);

        app.add_systems(
            Update,
            (add_ground_cosmetics, add_player_cosmetics).in_set(RendererSet),
        );
    }
}

fn add_ground_cosmetics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridBindlessMaterial>>,
    q_ground: Query<(Entity, &Ground), Without<ClientRenderer>>,
    game_assets: Res<GameAssets>,
) {
    for (entity, Ground { width, height }) in q_ground.iter() {
        let mesh = Plane3d::default().mesh().size(*width, *height).build();
        let material = GridBindlessMaterial::new(
            UVec2::new(*width as u32, *height as u32),
            game_assets.prototype_textures.clone(),
        );
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
