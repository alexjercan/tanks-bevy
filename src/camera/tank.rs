use crate::meth::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

pub mod prelude {
    pub use super::{
        TankCamera, TankCameraInput, TankCameraPlugin, TankCameraSet, TankCameraTransformTarget,
    };
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TankCamera {
    /// The initial focus point of the camera
    pub focus: Vec3,
    /// The orbit sensitivity of the camera
    pub orbit_sensitivity: f32,
    /// The zoom sensitivity of the camera
    pub zoom_sensitivity: f32,
    /// Minimum zoom distance
    pub min_zoom: f32,
    /// Maximum zoom distance
    pub max_zoom: f32,
    /// The minimum pitch
    pub min_pitch: f32,
    /// The maximum pitch
    pub max_pitch: f32,
    /// The orbit smoothing factor
    pub orbit_smooth: f32,
    /// The zoom smoothing factor
    pub zoom_smooth: f32,
    /// The transform smooth factor
    pub transform_smooth: f32,
}

impl Default for TankCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            orbit_sensitivity: 0.01,
            zoom_sensitivity: 2.5,
            min_zoom: 5.0,
            max_zoom: 20.0,
            min_pitch: 0.0,
            max_pitch: PI / 2.0,
            orbit_smooth: 0.1,
            zoom_smooth: 0.1,
            transform_smooth: 0.01,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TankCameraInput {
    pub orbit: Vec2,
    pub zoom: f32,
}

impl Default for TankCameraInput {
    fn default() -> Self {
        Self {
            orbit: Vec2::ZERO,
            zoom: 0.0,
        }
    }
}

impl TankCameraInput {
    fn has_input(&self) -> bool {
        self.orbit != Vec2::ZERO || self.zoom != 0.0
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TankCameraTransformTarget {
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct TankCameraTransform {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    radius: f32,
}

/// This set is used for the Tank camera systems, which run in the Update stage
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TankCameraSet;

/// This plugin is used to add a pan orbit camera for an Tank style game. It will allow rotation
/// rotation around ground point using right mouse click and zoom using mouse scroll wheel. It will
/// also ensure that the camera follows the tank target.
pub struct TankCameraPlugin;

impl Plugin for TankCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initialize_camera_transform,
                update_camera_target_orbit,
                update_camera_transform,
                sync_camera_transform,
            )
                .in_set(TankCameraSet)
                .chain(),
        );
    }
}

fn initialize_camera_transform(
    mut commands: Commands,
    q_camera: Query<
        (Entity, &Transform, &TankCamera),
        (
            Without<TankCameraTransformTarget>,
            Without<TankCameraTransform>,
        ),
    >,
) {
    for (entity, transform, camera) in q_camera.iter() {
        let comp_vec = transform.translation - camera.focus;
        let yaw = comp_vec.z.atan2(comp_vec.x);
        let radius = comp_vec.length().max(camera.min_zoom).min(camera.max_zoom);
        let pitch = (comp_vec.y / radius).asin();

        commands
            .entity(entity)
            .insert(TankCameraTransform {
                focus: camera.focus,
                yaw,
                pitch,
                radius,
            })
            .insert(TankCameraTransformTarget {
                focus: camera.focus,
                yaw,
                pitch,
                radius,
            });
    }
}

fn update_camera_target_orbit(
    mut q_camera: Query<(&TankCamera, &TankCameraInput, &mut TankCameraTransformTarget)>,
) {
    for (camera, input, mut camera_target) in q_camera.iter_mut() {
        if !input.has_input() {
            continue;
        }

        camera_target.yaw -= input.orbit.x * camera.orbit_sensitivity;

        camera_target.pitch = (camera_target.pitch + input.orbit.y * camera.orbit_sensitivity)
            .clamp(camera.min_pitch, camera.max_pitch);

        camera_target.radius = (camera_target.radius - input.zoom * camera.zoom_sensitivity)
            .clamp(camera.min_zoom, camera.max_zoom);
    }
}

fn update_camera_transform(
    mut q_camera: Query<(
        &mut TankCameraTransform,
        &TankCameraTransformTarget,
        &TankCamera,
    )>,
    time: Res<Time>,
) {
    for (mut transform, target, camera) in q_camera.iter_mut() {
        if transform.focus != target.focus {
            // TODO: Fix this
            // transform.focus = transform.focus.lerp_and_snap(
            //     target.focus,
            //     camera.transform_smooth,
            //     time.delta_secs(),
            // );
            transform.focus = target.focus;
        }

        if transform.yaw != target.yaw {
            transform.yaw =
                transform
                    .yaw
                    .lerp_and_snap(target.yaw, camera.orbit_smooth, time.delta_secs());
        }

        if transform.pitch != target.pitch {
            transform.pitch =
                transform
                    .pitch
                    .lerp_and_snap(target.pitch, camera.orbit_smooth, time.delta_secs());
        }

        if transform.radius != target.radius {
            transform.radius = transform.radius.lerp_and_snap(
                target.radius,
                camera.zoom_smooth,
                time.delta_secs(),
            );
        }
    }
}

fn sync_camera_transform(
    mut q_camera: Query<(&mut Transform, &TankCameraTransform), Changed<TankCameraTransform>>,
) {
    for (mut transform, camera) in q_camera.iter_mut() {
        let rotation = Quat::from_rotation_y(camera.yaw) * Quat::from_rotation_x(-camera.pitch);
        *transform = Transform::IDENTITY
            .with_translation(camera.focus + rotation * Vec3::new(0.0, 0.0, camera.radius))
            .with_rotation(rotation);
    }
}
