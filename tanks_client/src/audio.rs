use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_kira_audio::prelude::*;

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::AudioEffectsPlugin;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffectsPlugin;

impl Plugin for AudioEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AudioPlugin, SpatialAudioPlugin));

        app.add_systems(
            Update,
            (
                play_cannon_fired,
                play_shell_impact,
                play_player_died,
                destroy_audio,
            )
                .run_if(in_state(GameStates::Playing)),
        );
    }
}


fn play_cannon_fired(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut fired: EventReader<CannonFiredEvent>,
) {
    for event in fired.read() {
        let sound = audio.play(game_assets.cannon_fire.clone()).handle();

        commands.spawn((
            Name::new("CannonFireSound"),
            Transform::from_translation(event.position),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
        ));
    }
}

fn play_shell_impact(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut impacts: EventReader<ShellImpactEvent>,
) {
    for event in impacts.read() {
        let sound = audio.play(game_assets.shell_impact.clone()).handle();

        commands.spawn((
            Name::new("ShellImpactSound"),
            Transform::from_translation(**event),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
        ));
    }
}

fn play_player_died(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut deaths: EventReader<PlayerDiedEvent>,
) {
    for event in deaths.read() {
        let sound = audio.play(game_assets.death.clone()).handle();

        commands.spawn((
            Name::new("PlayerDiedSound"),
            Transform::from_translation(event.position),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
        ));
    }
}

fn destroy_audio(
    mut commands: Commands,
    q_audio: Query<(Entity, &SpatialAudioEmitter)>,
) {
    for (entity, audio) in q_audio.iter() {
        if audio.instances.is_empty() {
            commands.entity(entity).despawn();
        }
    }
}
