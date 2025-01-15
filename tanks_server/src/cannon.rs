//! Tank cannon components and systems

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::Replicated;

use network::prelude::*;
use utils::prelude::*;

pub mod prelude {
    pub use super::{TankCannon, TankCannonInput, TankCannonPlugin, TankCannonSet};
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform)]
pub struct TankCannon {
    /// The fire rate of the cannon (in seconds per shot)
    pub fire_rate_secs: f32,
    /// The speed of the shell
    pub shell_speed: f32,
    /// The offset of the cannon from the tank
    pub offset: Vec3,
}

impl Default for TankCannon {
    fn default() -> Self {
        Self {
            fire_rate_secs: 1.0,
            shell_speed: 25.0,
            offset: Vec3::new(0.0, 0.23, 0.6),
        }
    }
}

#[derive(Component, Clone, Debug)]
struct TankCannonState {
    /// The cooldown time remaining before we can fire again (in seconds)
    cooldown: Timer,
}

impl Default for TankCannonState {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TankCannonInput {
    pub fire: bool,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform)]
struct TankCannonShell {
    time_to_live: f32,
    damage: f32,
}

impl Default for TankCannonShell {
    fn default() -> Self {
        Self {
            time_to_live: 1.0,
            damage: 50.0,
        }
    }
}

#[derive(Component, Clone, Debug)]
struct TankCannonShellState {
    time_to_live: Timer,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TankCannonSet;

pub struct TankCannonPlugin;

impl Plugin for TankCannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initialize_cannon,
                cannon_fire,
                shell_update_time_to_live,
                shell_update_collision,
            )
                .in_set(TankCannonSet)
                .chain(),
        );
    }
}

fn initialize_cannon(
    mut commands: Commands,
    q_cannon: Query<(Entity, &TankCannon), Without<TankCannonState>>,
) {
    for (entity, _cannon) in q_cannon.iter() {
        commands
            .entity(entity)
            .insert(TankCannonState { ..default() });
    }
}

fn cannon_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut q_cannon: Query<(
        &mut TankCannonInput,
        &Transform,
        &TankCannon,
        &mut TankCannonState,
    )>,
) {
    for (mut input, transform, cannon, mut state) in q_cannon.iter_mut() {
        if state.cooldown.tick(time.delta()).finished() {
            if !input.fire {
                continue;
            }

            let shell = TankCannonShell::default();

            commands.spawn((
                Replicated,
                Name::new("TankCannonShell"),
                Transform::from_translation(
                    transform.translation + transform.rotation * cannon.offset,
                )
                .with_rotation(transform.rotation * Quat::from_rotation_x(FRAC_PI_2)),
                NetworkEntity,
                Shell,
                Collider::cylinder(0.1, 0.1),
                RigidBody::Dynamic,
                Velocity {
                    linvel: transform.rotation * Vec3::Z * cannon.shell_speed + Vec3::Y * 2.0,
                    ..default()
                },
                shell,
                TankCannonShellState {
                    time_to_live: Timer::from_seconds(shell.time_to_live, TimerMode::Once),
                },
                ActiveEvents::COLLISION_EVENTS,
            ));

            state.cooldown = Timer::from_seconds(cannon.fire_rate_secs, TimerMode::Once);
        }

        input.fire = false;
    }
}

fn shell_update_time_to_live(
    time: Res<Time>,
    mut commands: Commands,
    mut q_shell: Query<(Entity, &TankCannonShell, &mut TankCannonShellState)>,
) {
    for (entity, _shell, mut state) in q_shell.iter_mut() {
        if state.time_to_live.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn shell_update_collision(
    mut commands: Commands,
    q_shell: Query<(Entity, &TankCannonShell, &CollisionWith)>,
) {
    for (entity, shell, collision_with) in q_shell.iter() {
        commands.entity(entity).despawn_recursive();
        commands.entity(collision_with.entity).insert(Damage {
            amount: shell.damage,
        });
    }
}
