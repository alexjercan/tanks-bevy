//! Health component and system

use bevy::prelude::*;

pub mod prelude {
    pub use super::{Damage, Dead, Health, HealthPlugin, HealthSet};
}

#[derive(Component, Clone, Debug)]
pub struct Health {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Damage {
    pub amount: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Dead;

impl Default for Health {
    fn default() -> Self {
        Self { value: 100.0 }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HealthSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_damage);
    }
}

fn handle_damage(
    mut commands: Commands,
    mut q_health: Query<(Entity, &mut Health, &Damage), Without<Dead>>,
) {
    for (entity, mut health, damage) in q_health.iter_mut() {
        health.value -= damage.amount;
        if health.value <= 0.0 {
            commands.entity(entity).insert(Dead);
        }

        commands.entity(entity).remove::<Damage>();
    }
}
