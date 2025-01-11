use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

pub mod prelude {
    pub use super::{TankController, TankControllerInput, TankControllerPlugin, TankControllerSet};
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TankController {
    /// The movement speed of the tank
    pub move_speed: f32,
    /// The rotation speed of the tank
    pub rotation_speed: f32,
    /// Acceleration of the tank (in m/s^2)
    pub acceleration: f32,
    /// Deceleration of the tank (in m/s^2)
    pub deceleration: f32,
}

impl Default for TankController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            rotation_speed: 2.0,
            acceleration: 5.0,
            deceleration: 20.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct TankControllerState {
    vertical_movement: f32,
    y_rotation: f32,
    speed: f32,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TankControllerInput {
    pub forward: f32,
    pub steer: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TankControllerSet;

pub struct TankControllerPlugin;

impl Plugin for TankControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            initialize_controller_state.in_set(TankControllerSet),
        )
        .add_systems(FixedUpdate, update_controller.in_set(TankControllerSet));
    }
}

fn initialize_controller_state(
    mut commands: Commands,
    q_controller: Query<(Entity, &TankController), Without<TankControllerState>>,
) {
    for (entity, _tank) in q_controller.iter() {
        commands.entity(entity).insert(TankControllerState {
            vertical_movement: 0.0,
            y_rotation: 0.0,
            speed: 0.0,
        });
    }
}

fn update_controller(
    time: Res<Time>,
    mut q_controller: Query<(
        &TankController,
        &TankControllerInput,
        &mut TankControllerState,
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
) {
    for (tank, input, mut state, mut transform, mut controller, output) in q_controller.iter_mut() {
        let delta_time = time.delta_secs();

        let accelerating =
            input.forward != 0.0 && (state.speed == 0.0 || state.speed.signum() == input.forward);
        if accelerating {
            state.speed = tank
                .move_speed
                .min(state.speed.abs() + tank.acceleration * delta_time)
                * input.forward;
        } else {
            state.speed = (state.speed.abs() - tank.deceleration * delta_time).max(0.0)
                * state.speed.signum();
        }

        let mut movement = Vec3::new(0.0, 0.0, state.speed);

        if output.map(|o| o.grounded).unwrap_or(false) {
            state.vertical_movement = 0.0;
        }

        if input.steer != 0.0 {
            state.y_rotation = (state.y_rotation + input.steer * tank.rotation_speed * delta_time)
                .rem_euclid(2.0 * PI);
        }

        movement.y = state.vertical_movement;
        state.vertical_movement += -9.81 * delta_time * controller.custom_mass.unwrap_or(1.0);
        controller.translation = Some(transform.rotation * (movement * delta_time));

        transform.rotation = Quat::from_rotation_y(state.y_rotation);
    }
}
