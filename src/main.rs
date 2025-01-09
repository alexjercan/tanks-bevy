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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TankCameraPlugin)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .run();
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
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
