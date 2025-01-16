use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_kira_audio::prelude::*;

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::AudioEffectsPlugin;
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
struct EngineSound;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffectsPlugin;

#[derive(Resource, Component, Default, Clone)]
struct ExplosionChannel;

#[derive(Resource, Component, Default, Clone)]
struct EngineChannel;

impl Plugin for AudioEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AudioPlugin, SpatialAudioPlugin));

        app.add_systems(
            Update,
            (
                play_cannon_fired,
                play_shell_impact,
                play_player_died,
                play_engine_sound,
                destroy_audio,
            )
                .run_if(in_state(GameStates::Playing)),
        );

        app
            .add_audio_channel::<ExplosionChannel>()
            .add_audio_channel::<EngineChannel>();
    }
}


fn play_cannon_fired(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    channel: Res<AudioChannel<ExplosionChannel>>,
    mut fired: EventReader<CannonFiredEvent>,
) {
    for event in fired.read() {
        let sound = channel.play(game_assets.cannon_fire.clone()).handle();

        commands.spawn((
            Name::new("CannonFireSound"),
            Transform::from_translation(event.position),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
            StateScoped(GameStates::Playing),
        ));
    }
}

fn play_shell_impact(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    channel: Res<AudioChannel<ExplosionChannel>>,
    mut impacts: EventReader<ShellImpactEvent>,
) {
    for event in impacts.read() {
        let sound = channel.play(game_assets.shell_impact.clone()).handle();

        commands.spawn((
            Name::new("ShellImpactSound"),
            Transform::from_translation(**event),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
            StateScoped(GameStates::Playing),
        ));
    }
}

fn play_player_died(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    channel: Res<AudioChannel<ExplosionChannel>>,
    mut deaths: EventReader<PlayerDiedEvent>,
) {
    for event in deaths.read() {
        let sound = channel.play(game_assets.death.clone()).handle();

        commands.spawn((
            Name::new("PlayerDiedSound"),
            Transform::from_translation(event.position),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 50.0 },
            StateScoped(GameStates::Playing),
        ));
    }
}

fn play_engine_sound(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    channel: Res<AudioChannel<EngineChannel>>,
    q_player: Query<Entity, (With<Player>, Without<EngineSound>)>,
) {
    for entity in q_player.iter() {
        let sound = channel.play(game_assets.tank_engine.clone()).looped().handle();

        commands.entity(entity)
            .insert(EngineSound)
            .with_child((
                Name::new("EngineSound"),
                Transform::default(),
                SpatialAudioEmitter {
                    instances: vec![sound],
                },
                SpatialRadius { radius: 25.0 },
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
