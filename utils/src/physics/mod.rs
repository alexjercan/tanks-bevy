//! Physics collision plugin

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod prelude {
    pub use super::{CollisionPlugin, CollisionSet, CollisionWith};
}

#[derive(Component, Clone, Copy, Debug)]
pub struct CollisionWith {
    pub entity: Entity,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSet;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_collision_events).in_set(CollisionSet).chain(),
        );
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(other, entity, _) => {
                commands
                    .entity(*entity)
                    .insert(CollisionWith { entity: *other });
                commands
                    .entity(*other)
                    .insert(CollisionWith { entity: *entity });
            }
            CollisionEvent::Stopped(_other, _entity, _) => {}
        }
    }
}
