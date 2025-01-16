use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use serde::{Deserialize, Serialize};

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
                handle_engine_sound,
                destroy_audio,
            )
                .run_if(in_state(GameStates::Playing)),
        );

        app.add_audio_channel::<ExplosionChannel>();
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
    mut audio: ResMut<DynamicAudioChannels>,
    q_player: Query<(Entity, &Player), Without<EngineSound>>,
) {
    for (entity, Player { client_id, .. }) in q_player.iter() {
        let sound = audio
            .create_channel(&client_id.get().to_string())
            .play(game_assets.tank_engine.clone())
            .looped()
            .handle();

        commands.entity(entity).insert(EngineSound).with_child((
            Name::new("EngineSound"),
            Transform::default(),
            SpatialAudioEmitter {
                instances: vec![sound],
            },
            SpatialRadius { radius: 25.0 },
        ));
    }
}

fn handle_engine_sound(
    audio: Res<DynamicAudioChannels>,
    q_player: Query<(&Player, &Throttle), With<EngineSound>>,
) {
    for (Player { client_id, .. }, Throttle { value }) in q_player.iter() {
        let value = value.clamp(0.0, 1.0) as f64;
        let pitch = 1.0.lerp(1.5, value);
        audio
            .channel(&client_id.get().to_string())
            .set_playback_rate(pitch);
    }
}

fn destroy_audio(mut commands: Commands, q_audio: Query<(Entity, &SpatialAudioEmitter)>) {
    for (entity, audio) in q_audio.iter() {
        if audio.instances.is_empty() {
            commands.entity(entity).despawn();
        }
    }
}
