//! Despawn component and system

use bevy::prelude::*;

pub mod prelude {
    pub use super::{DespawnAfter, DespawnAfterPlugin, DespawnAfterSet};
}

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct DespawnAfter(pub Timer);

impl DespawnAfter {
    pub fn new(time: f32) -> Self {
        Self(Timer::from_seconds(time, TimerMode::Once))
    }
}

#[derive(Component, Clone, Debug)]
pub struct Health {
    pub value: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DespawnAfterSet;

pub struct DespawnAfterPlugin;

impl Plugin for DespawnAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after.in_set(DespawnAfterSet));
    }
}

fn despawn_after(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnAfter)>,
) {
    for (entity, mut despawn_after) in query.iter_mut() {
        if despawn_after.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
