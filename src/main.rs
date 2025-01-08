use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/tank.glb#Scene0")]
    tank: Handle<Scene>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .run();
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>, scenes: Res<Assets<Scene>>) {
    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        SceneRoot(game_assets.tank.clone()),
        Transform::from_xyz(
            0.0,
            0.5,
            0.0,
        ),
    ));
}
