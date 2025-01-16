use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
};
use bevy_kira_audio::prelude::*;

use crate::prelude::*;
use network::prelude::*;
use utils::prelude::*;

pub mod prelude {
    pub use super::TankCameraPlugin;
}

pub struct TankCameraPlugin;

impl Plugin for TankCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OrbiterTransformPlugin);
        app.add_plugins(SmoothTransformPlugin);

        app.configure_sets(
            Update,
            OrbiterTransformSet.run_if(in_state(GameStates::Playing)),
        );
        app.configure_sets(
            PostUpdate,
            SmoothTransformSet.run_if(in_state(GameStates::Playing)),
        );

        app.add_systems(OnEnter(GameStates::Playing), spawn_camera);
        app.add_systems(
            Update,
            update_camera_target.run_if(in_state(GameStates::Playing)),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Name::new("CameraRoot"),
            SmoothTransform::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            StateScoped(GameStates::Playing),
        ))
        .with_child((
            Name::new("Camera3d"),
            OrbiterTransform::default(),
            Camera {
                hdr: true,
                clear_color: Color::BLACK.into(),
                ..default()
            },
            Camera3d::default(),
            Tonemapping::None,
            Bloom {
                intensity: 0.2,
                ..default()
            },
            Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            SpatialAudioReceiver,
        ));
}

fn update_camera_target(
    // TODO: somehow get local player without all the network stuff
    q_player: Query<(&Player, &Transform)>,
    local_player: Res<LocalPlayer>,
    mut q_smooth: Query<&mut SmoothTransform>,
) {
    for (Player { client_id, .. }, transform) in q_player.iter() {
        if *client_id == **local_player {
            for mut target in q_smooth.iter_mut() {
                target.target = transform.translation;
            }
        }
    }
}
