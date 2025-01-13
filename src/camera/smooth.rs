//! The orbiter transform plugin

use bevy::prelude::*;

// use crate::meth::prelude::*;

pub mod prelude {
  pub use super::{SmoothTransform, SmoothTransformPlugin, SmoothTransformSet};
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform)]
pub struct SmoothTransform {
    /// The target position of the transform
    pub target: Vec3,
    /// The translate smooth factor
    pub translate_smooth: f32,
}

impl Default for SmoothTransform {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            translate_smooth: 0.01,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct SmoothTransformState {
    /// The current position of the transform
    position: Vec3,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmoothTransformSet;

pub struct SmoothTransformPlugin;

impl Plugin for SmoothTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initialize_smooth,
                update_smooth_transform,
                sync_smooth_transform,
            )
                .in_set(SmoothTransformSet)
                .chain(),
        );
    }
}

fn initialize_smooth(
    mut commands: Commands,
    q_smooth: Query<
        (Entity, &SmoothTransform),
        (
            Without<SmoothTransformState>,
        ),
    >,
) {
    for (entity, SmoothTransform { target, .. }) in q_smooth.iter() {
        commands
            .entity(entity)
            .insert(SmoothTransformState {
                position: *target,
            });
    }
}

fn update_smooth_transform(
    // time: Res<Time>,
    mut q_smooth: Query<(&mut SmoothTransformState, &SmoothTransform)>,
) {
    for (mut state, SmoothTransform { target, .. }) in q_smooth.iter_mut() {
        // TODO: Fix lerp_and_snap
        // state.position = state.position.lerp_and_snap(*target, *translate_smooth, time.delta_secs());
        state.position = *target;
    }
}

fn sync_smooth_transform(
    mut q_smooth: Query<(&mut Transform, &SmoothTransformState, &SmoothTransform), Changed<SmoothTransformState>>,
) {
    for (mut transform, SmoothTransformState { position }, SmoothTransform { .. }) in q_smooth.iter_mut() {
        transform.translation = *position;
    }
}
