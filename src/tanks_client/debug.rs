use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use iyes_perf_ui::prelude::*;

pub mod prelude {
    pub use super::{DebugPlugin, DebugSet};
}

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
struct ShowAxes(pub bool);

/// System set for the debug plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugSet;

/// This plugin adds a simple debug system that toggles the Perf UI
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // we want Bevy to measure these values for us:
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .init_resource::<ShowAxes>()
            .add_plugins(PerfUiPlugin)
            // Bevy egui inspector
            .add_plugins(WorldInspectorPlugin::new())
            // we want to show Rapier debug information:
            .add_plugins(RapierDebugRenderPlugin::default())
            // We need to order our system before PerfUiSet::Setup,
            // so that iyes_perf_ui can process any new Perf UI in the same
            // frame as we spawn the entities. Otherwise, Bevy UI will complain.
            .add_systems(Update, toggle.before(iyes_perf_ui::PerfUiSet::Setup))
            .add_systems(Update, draw_axes)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));
}

fn toggle(
    mut commands: Commands,
    q_root: Query<Entity, With<PerfUiRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut show_axes: ResMut<ShowAxes>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));
        }

        show_axes.0 = !show_axes.0;
    }
}

// This system draws the axes based on the cube's transform, with length based on the size of
// the entity's axis-aligned bounding box (AABB).
fn draw_axes(mut gizmos: Gizmos, query: Query<&Transform>, show_axes: Res<ShowAxes>) {
    if !show_axes.0 {
        return;
    }

    for &transform in &query {
        let length = 3.0;
        gizmos.axes(transform, length);
    }
}
