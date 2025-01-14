use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::network::prelude::PlayerInputEvent;
use crate::client::prelude::*;

pub mod prelude {
    pub use super::TankInputPlugin;
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum PlayerInputAction {
    #[actionlike(DualAxis)]
    Move,
}

impl PlayerInputAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default().with_dual_axis(PlayerInputAction::Move, VirtualDPad::wasd())
    }
}

#[derive(Component, Clone, Debug, Copy, Deref, DerefMut, Default)]
struct PlayerInputMove(Vec2);

pub struct TankInputPlugin;

impl Plugin for TankInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerInputAction>::default());
        app.add_systems(
            OnEnter(GameStates::Playing),
            spawn_input,
        );
        app.add_systems(
            Update,
            (update_player_input).run_if(in_state(GameStates::Playing)),
        );
    }
}

fn spawn_input(
    mut commands: Commands,
) {
    commands.spawn((
        Name::new("PlayerInput"),
        PlayerInputMove::default(),
        InputManagerBundle::with_map(PlayerInputAction::default_input_map()),
        StateScoped(GameStates::Playing),
    ));
}

fn update_player_input(
    mut input: EventWriter<PlayerInputEvent>,
    mut q_input: Query<(&mut PlayerInputMove, &ActionState<PlayerInputAction>)>,
) {
    for (mut prev, action) in q_input.iter_mut() {
        let movement = action.clamped_axis_pair(&PlayerInputAction::Move);

        if movement.x != prev.x || movement.y != prev.y {
            **prev = movement;
            input.send(PlayerInputEvent(movement));
        }
    }
}
