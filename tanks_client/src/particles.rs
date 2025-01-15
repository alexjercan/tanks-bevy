use bevy::{prelude::*, render::mesh::{SphereKind, SphereMeshBuilder}};
use bevy_hanabi::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use network::prelude::*;

pub mod prelude {
    pub use super::ParticleEffectsPlugin;
}

#[derive(Resource)]
struct ParticleSystems {
    explosion: Handle<EffectAsset>,
    smoke: Handle<EffectAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEffectsPlugin;

impl Plugin for ParticleEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin);

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                play_cannon_fired,
                play_shell_impact,
                play_player_died,
            )
                .run_if(in_state(GameStates::Playing))
                .run_if(resource_exists::<ParticleSystems>),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.6, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.05));
    size_gradient1.add_key(0.3, Vec3::splat(0.05));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.8).normal(writer.lit(1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -16.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(4.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(60.)).expr(),
    };

    // Clear the trail velocity so trail particles just stay in place as they fade
    // away
    let init_vel_trail =
        SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());

    let lead = ParticleGroupSet::single(0);
    let trail = ParticleGroupSet::single(1);

    let effect = EffectAsset::new(
        // 2k lead particles, with 32 trail particles each
        2048,
        Spawner::once(2048.0.into(), true),
        writer.finish(),
    )
    // Tie together trail particles to make arcs. This way we don't need a lot of them, yet there's
    // a continuity between them.
    .with_ribbons(2048 * 32, 1.0 / 64.0, 0.2, 0)
    .with_name("Explosion")
    .init_groups(init_pos, lead)
    .init_groups(init_vel, lead)
    .init_groups(init_age, lead)
    .init_groups(init_lifetime, lead)
    .init_groups(init_vel_trail, trail)
    .update_groups(update_drag, lead)
    .update_groups(update_accel, lead)
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient1.clone(),
        },
        lead,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient1.clone(),
            screen_space_size: false,
        },
        lead,
    )
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient1,
        },
        trail,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        },
        trail,
    );

    let explosion_handle = effects.add(effect);

    // Smoke
    let mesh = meshes.add(SphereMeshBuilder::new(0.5, SphereKind::Ico { subdivisions: 4 }).build());

    let writer = ExprWriter::new();

    // Position the particle laterally within a small radius.
    let init_xz_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(1.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Position the particle vertically. Jiggle it a little bit for variety's
    // sake.
    let init_y_pos = SetAttributeModifier::new(
        Attribute::POSITION,
        writer
            .attr(Attribute::POSITION)
            .add(writer.rand(VectorType::VEC3F) * writer.lit(Vec3::new(0.0, 1.0, 0.0)))
            .expr(),
    );

    // Set up the age and lifetime.
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(3.0).expr());

    // Vary the size a bit.
    let init_size = SetAttributeModifier::new(
        Attribute::F32_0,
        (writer.rand(ScalarType::Float) * writer.lit(2.0) + writer.lit(0.5)).expr(),
    );

    // Make the particles move backwards at a constant speed.
    let init_velocity = SetAttributeModifier::new(
        Attribute::VELOCITY,
        writer.lit(Vec3::new(0.0, 0.0, -20.0)).expr(),
    );

    // Make the particles shrink over time.
    let update_size = SetAttributeModifier::new(
        Attribute::SIZE,
        writer
            .attr(Attribute::F32_0)
            .mul(
                writer
                    .lit(1.0)
                    .sub((writer.attr(Attribute::AGE)).mul(writer.lit(0.75)))
                    .max(writer.lit(0.0)),
            )
            .expr(),
    );

    let module = writer.finish();

    let effect = EffectAsset::new(256, Spawner::once(16.0.into(), true), module)
        .with_name("Smoke")
        .init(init_xz_pos)
        .init(init_y_pos)
        .init(init_age)
        .init(init_lifetime)
        .init(init_size)
        .init(init_velocity)
        .update(update_size)
        .mesh(mesh);

    // Add the effect.
    let smoke_handle = effects.add(effect);

    commands.insert_resource(
        ParticleSystems {
            explosion: explosion_handle,
            smoke: smoke_handle,
        }
    );
}

fn play_cannon_fired(
    mut commands: Commands,
    mut fired: EventReader<CannonFiredEvent>,
    particle_systems: Res<ParticleSystems>,
) {
    for event in fired.read() {
        commands.spawn((
            Name::new("Smoke"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(particle_systems.smoke.clone()),
                transform: Transform::from_translation(**event),
                ..Default::default()
            },
        ));
    }
}

fn play_shell_impact(
    mut commands: Commands,
    mut impacts: EventReader<ShellImpactEvent>,
    particle_systems: Res<ParticleSystems>,
) {
    for event in impacts.read() {
        /*
        commands.spawn((
            Name::new("firework"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(particle_systems.explosion.clone()),
                transform: Transform::from_translation(**event),
                ..Default::default()
            },
        ));
        */
    }
}

fn play_player_died(
    mut commands: Commands,
    mut deaths: EventReader<PlayerDiedEvent>,
    particle_systems: Res<ParticleSystems>,
) {
    for event in deaths.read() {
        commands.spawn((
            Name::new("Explosion"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(particle_systems.explosion.clone()),
                transform: Transform::from_translation(event.position),
                ..Default::default()
            },
        ));
    }
}
