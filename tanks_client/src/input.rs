use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::TankInputPlugin;
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum PlayerInputAction {
    #[actionlike(DualAxis)]
    Move,
    Fire,
}

impl PlayerInputAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with_dual_axis(PlayerInputAction::Move, VirtualDPad::wasd())
            .with(Self::Fire, KeyCode::Space)
    }
}

#[derive(Component, Clone, Debug, Copy, Deref, DerefMut, Default)]
struct PlayerInputMove(Vec2);

pub struct TankInputPlugin;

impl Plugin for TankInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerInputAction>::default());
        app.add_systems(OnEnter(GameStates::Playing), spawn_input);
        app.add_systems(
            Update,
            (update_player_input)
                .run_if(in_state(GameStates::Playing))
                .run_if(resource_exists::<LocalPlayerEntity>),
        );
        app.add_systems(
            Update,
            (update_player_spawn)
                .run_if(in_state(GameStates::Playing))
                .run_if(not(resource_exists::<LocalPlayerEntity>)),
        );
    }
}

fn spawn_input(mut commands: Commands) {
    commands.spawn((
        Name::new("PlayerInput"),
        PlayerInputMove::default(),
        InputManagerBundle::with_map(PlayerInputAction::default_input_map()),
        StateScoped(GameStates::Playing),
    ));
}

fn update_player_input(
    mut input: EventWriter<PlayerInputEvent>,
    mut fire: EventWriter<PlayerFireEvent>,
    mut q_input: Query<(&mut PlayerInputMove, &ActionState<PlayerInputAction>)>,
) {
    for (mut prev, action) in q_input.iter_mut() {
        let movement = action.clamped_axis_pair(&PlayerInputAction::Move);

        if movement.x != prev.x || movement.y != prev.y {
            **prev = movement;
            input.send(PlayerInputEvent(movement));
        }

        if action.just_pressed(&PlayerInputAction::Fire) {
            fire.send(PlayerFireEvent);
        }
    }
}

fn update_player_spawn(
    mut spawn: EventWriter<PlayerSpawnEvent>,
    mut q_input: Query<&ActionState<PlayerInputAction>>,
) {
    for action in q_input.iter_mut() {
        if action.just_pressed(&PlayerInputAction::Fire) {
            spawn.send(PlayerSpawnEvent);
        }
    }
}
