use bevy_asset_loader::prelude::*;
use bevy::prelude::*;
use tanks::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    tank: Handle<Scene>,
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
struct TankController {

}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TankCameraPlugin)
        .add_plugins(GridMaterialPlugin)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridBindlessMaterial>>,
    game_assets: Res<GameAssets>
) {
    // Ground
    let size = 32;
    let mesh = Plane3d::default().mesh().size(size as f32, size as f32).build();
    let material = GridBindlessMaterial::new(UVec2::new(size, size), game_assets.prototype_textures.clone());
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(Vec3::ZERO),
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        TankCamera {
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        TankCameraTarget::default(),
        SceneRoot(game_assets.tank.clone()),
        Transform::from_xyz(
            0.0,
            0.5,
            0.0,
        ),
    ));
}
