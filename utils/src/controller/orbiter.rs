//! The orbiter transform plugin

use std::f32::consts::PI;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::meth::prelude::*;

pub mod prelude {
    pub use super::{OrbiterTransform, OrbiterTransformPlugin, OrbiterTransformSet};
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform)]
pub struct OrbiterTransform {
    /// The initial focus point of the transform
    pub focus: Vec3,
    /// The orbit sensitivity of the transform
    pub orbit_sensitivity: f32,
    /// The zoom sensitivity of the transform
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
}

impl Default for OrbiterTransform {
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
        }
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum OrbiterTransformAction {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(Axis)]
    Zoom,
}

impl OrbiterTransformAction {
    // TODO: Allow customization of the input map
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with_dual_axis(OrbiterTransformAction::Pan, MouseMove::default())
            .with_axis(OrbiterTransformAction::Zoom, MouseScrollAxis::Y)
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct OrbiterTransformTarget {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct OrbiterTransformState {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrbiterTransformSet;

pub struct OrbiterTransformPlugin;

impl Plugin for OrbiterTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<OrbiterTransformAction>::default());

        app.add_systems(
            Update,
            (
                initialize_orbiter,
                update_orbiter_target,
                update_orbiter_transform,
                sync_orbiter_transform,
            )
                .in_set(OrbiterTransformSet)
                .chain(),
        );
    }
}

fn initialize_orbiter(
    mut commands: Commands,
    q_camera: Query<
        (Entity, &Transform, &OrbiterTransform),
        (
            Without<OrbiterTransformTarget>,
            Without<OrbiterTransformState>,
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
            .insert(OrbiterTransformTarget { yaw, pitch, radius })
            .insert(OrbiterTransformState { yaw, pitch, radius })
            .insert(InputManagerBundle::with_map(
                OrbiterTransformAction::default_input_map(),
            ));
    }
}

fn update_orbiter_target(
    mut q_camera: Query<(
        &OrbiterTransform,
        &ActionState<OrbiterTransformAction>,
        &mut OrbiterTransformTarget,
    )>,
) {
    for (camera, action, mut camera_target) in q_camera.iter_mut() {
        let zoom = action.value(&OrbiterTransformAction::Zoom);
        let orbit = action.axis_pair(&OrbiterTransformAction::Pan);

        camera_target.yaw -= orbit.x * camera.orbit_sensitivity;

        camera_target.pitch = (camera_target.pitch + orbit.y * camera.orbit_sensitivity)
            .clamp(camera.min_pitch, camera.max_pitch);

        camera_target.radius = (camera_target.radius - zoom * camera.zoom_sensitivity)
            .clamp(camera.min_zoom, camera.max_zoom);
    }
}

fn update_orbiter_transform(
    time: Res<Time>,
    mut q_camera: Query<(
        &mut OrbiterTransformState,
        &OrbiterTransformTarget,
        &OrbiterTransform,
    )>,
) {
    for (mut state, target, camera) in q_camera.iter_mut() {
        if state.yaw != target.yaw {
            state.yaw = state
                .yaw
                .lerp_and_snap(target.yaw, camera.orbit_smooth, time.delta_secs());
        }

        if state.pitch != target.pitch {
            state.pitch =
                state
                    .pitch
                    .lerp_and_snap(target.pitch, camera.orbit_smooth, time.delta_secs());
        }

        if state.radius != target.radius {
            state.radius =
                state
                    .radius
                    .lerp_and_snap(target.radius, camera.zoom_smooth, time.delta_secs());
        }
    }
}

fn sync_orbiter_transform(
    mut q_camera: Query<
        (&mut Transform, &OrbiterTransformState, &OrbiterTransform),
        Changed<OrbiterTransformState>,
    >,
) {
    for (mut transform, state, camera) in q_camera.iter_mut() {
        let rotation = Quat::from_rotation_y(state.yaw) * Quat::from_rotation_x(-state.pitch);
        *transform = Transform::IDENTITY
            .with_translation(camera.focus + rotation * Vec3::new(0.0, 0.0, state.radius))
            .with_rotation(rotation);
    }
}
