mod diagnostics_plugin;
mod span_recorder;

use crate::collision::{ColliderMarker, Collisions};
use crate::position::Position;
use crate::prelude::{PhysicsStepSet, RigidBody};
use crate::schedule::PhysicsSchedule;
use bevy::prelude::{PostUpdate, Query, Res, Without};
use bevy::{
    app::{App, Plugin},
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::IntoSystemSetConfigs,
    prelude::{IntoSystemConfigs, ResMut, Resource, SystemSet},
};
use span_recorder::{PhysicsSpan, PhysicsSpanRecorder};

pub struct PhysicsDiagnosticsPlugin;

impl PhysicsDiagnosticsPlugin {
    pub const BROAD_TIME: DiagnosticPath = DiagnosticPath::const_new("avian/broad_time");
    pub const NARROW_TIME: DiagnosticPath = DiagnosticPath::const_new("avian/narrow_time");
    pub const SOLVER_TIME: DiagnosticPath = DiagnosticPath::const_new("avian/solver_time");
    pub const REPORT_TIME: DiagnosticPath = DiagnosticPath::const_new("avian/report_time");
    pub const SPATIAL_TIME: DiagnosticPath = DiagnosticPath::const_new("avian/spatial_time");
    pub const RIGIDS_COUNT: DiagnosticPath = DiagnosticPath::const_new("avian/rigids_count");
    pub const COLLIDERS_COUNT: DiagnosticPath = DiagnosticPath::const_new("avian/colliders_count");
    pub const CONTACTS_COUNT: DiagnosticPath = DiagnosticPath::const_new("avian/contacts_count");
}

impl Plugin for PhysicsDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsSpanRecorder>()
            .register_diagnostic(Diagnostic::new(Self::BROAD_TIME).with_suffix("ms"))
            .register_diagnostic(Diagnostic::new(Self::NARROW_TIME).with_suffix("ms"))
            .register_diagnostic(Diagnostic::new(Self::SOLVER_TIME).with_suffix("ms"))
            .register_diagnostic(Diagnostic::new(Self::SPATIAL_TIME).with_suffix("ms"))
            .register_diagnostic(Diagnostic::new(Self::REPORT_TIME).with_suffix("ms"))
            .register_diagnostic(Diagnostic::new(Self::RIGIDS_COUNT))
            .register_diagnostic(Diagnostic::new(Self::COLLIDERS_COUNT))
            .register_diagnostic(Diagnostic::new(Self::CONTACTS_COUNT))
            .add_systems(
                PhysicsSchedule,
                diagnostics_frame_start_system.in_set(PhysicsStepSet::DiagnosticsInitialise),
            )
            .add_systems(
                PhysicsSchedule,
                diagnostics_frame_end_system.in_set(PhysicsStepSet::DiagnosticsFinalise),
            )
            .add_systems(PostUpdate, diagnostic_counts_system);

        for (span_type, start_set, end_set) in vec![
            (
                PhysicsSpan::BroadPhase,
                PhysicsStepSet::PreBroadPhase,
                PhysicsStepSet::PostBroadPhase,
            ),
            (
                PhysicsSpan::NarrowPhase,
                PhysicsStepSet::PreNarrowPhase,
                PhysicsStepSet::PostNarrowPhase,
            ),
            (
                PhysicsSpan::Solver,
                PhysicsStepSet::PreSolver,
                PhysicsStepSet::PostSolver,
            ),
            (
                PhysicsSpan::ReportContacts,
                PhysicsStepSet::PreReportContacts,
                PhysicsStepSet::PostReportContacts,
            ),
            (
                PhysicsSpan::SpatialQueries,
                PhysicsStepSet::PreSpatialQuery,
                PhysicsStepSet::PostSpatialQuery,
            ),
        ] {
            app.add_systems(
                PhysicsSchedule,
                start_span_wrapper(span_type).in_set(start_set),
            )
            .add_systems(PhysicsSchedule, end_span_wrapper(span_type).in_set(end_set));
        }
    }
}

fn start_span_wrapper(span_type: PhysicsSpan) -> impl FnMut(ResMut<PhysicsSpanRecorder>) {
    move |mut diagnostic_recorder: ResMut<PhysicsSpanRecorder>| {
        diagnostic_recorder.start_span(span_type);
    }
}

fn end_span_wrapper(span_type: PhysicsSpan) -> impl FnMut(ResMut<PhysicsSpanRecorder>) {
    move |mut diagnostic_recorder: ResMut<PhysicsSpanRecorder>| {
        diagnostic_recorder.end_span(span_type);
    }
}

fn diagnostics_frame_start_system(mut diagnostic_recorder: ResMut<PhysicsSpanRecorder>) {
    diagnostic_recorder.reset();
}

fn diagnostics_frame_end_system(
    mut diagnostic_recorder: ResMut<PhysicsSpanRecorder>,
    mut diagnostics: Diagnostics,
) {
    diagnostic_recorder.finalise(&mut diagnostics);
}

fn diagnostic_counts_system(
    mut diagnostics: Diagnostics,
    rigid_bodies_query: Query<&RigidBody>,
    colliders_query: Query<(&ColliderMarker)>,
    collisions: Res<Collisions>,
) {
    diagnostics.add_measurement(&PhysicsDiagnosticsPlugin::RIGIDS_COUNT, || {
        rigid_bodies_query.iter().count() as f64
    });
    diagnostics.add_measurement(&PhysicsDiagnosticsPlugin::COLLIDERS_COUNT, || {
        colliders_query.iter().count() as f64
    });
    diagnostics.add_measurement(&PhysicsDiagnosticsPlugin::CONTACTS_COUNT, || {
        collisions.iter().count() as f64
    });
}
