use bevy::prelude::*;

use crate::{
    DawnStarted, DayNightConfig, DayNightPlugin, DayNightSystems, DayStarted, DuskStarted,
    ManagedLightConfig, Moon, NightStarted, Sun,
};

#[derive(Resource, Default)]
struct PhaseCounts {
    dawn: u32,
    day: u32,
    dusk: u32,
    night: u32,
}

fn count_phase_messages(
    mut counts: ResMut<PhaseCounts>,
    mut dawn: MessageReader<DawnStarted>,
    mut day: MessageReader<DayStarted>,
    mut dusk: MessageReader<DuskStarted>,
    mut night: MessageReader<NightStarted>,
) {
    counts.dawn += dawn.read().count() as u32;
    counts.day += day.read().count() as u32;
    counts.dusk += dusk.read().count() as u32;
    counts.night += night.read().count() as u32;
}

#[test]
fn plugin_builds_and_initializes_resources() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, DayNightPlugin::default()));
    app.update();

    assert!(app.world().contains_resource::<crate::TimeOfDay>());
    assert!(app.world().contains_resource::<crate::CelestialState>());
    assert!(app.world().contains_resource::<crate::DayNightLighting>());
    assert!(
        app.world()
            .contains_resource::<crate::DayNightDiagnostics>()
    );
}

#[test]
fn forward_jump_emits_each_crossed_phase_once() {
    let mut app = App::new();
    let config = DayNightConfig {
        initial_time: 17.5,
        paused: true,
        ..default()
    };

    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));
    app.insert_resource(PhaseCounts::default());
    app.add_systems(
        Update,
        count_phase_messages.after(DayNightSystems::DetectPhaseTransitions),
    );

    app.update();
    app.world_mut()
        .resource_mut::<DayNightConfig>()
        .queue_advance_hours(15.0);
    app.update();

    let counts = app.world().resource::<PhaseCounts>();
    assert_eq!(counts.dusk, 1);
    assert_eq!(counts.night, 1);
    assert_eq!(counts.dawn, 1);
    assert_eq!(counts.day, 1);
}

#[test]
fn plugin_supports_existing_managed_lights() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, DayNightPlugin::default()));
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Name::new("Test Sun"),
            Sun,
            DirectionalLight::default(),
            Transform::default(),
        ));
        commands.spawn((
            Name::new("Test Moon"),
            Moon,
            DirectionalLight::default(),
            Transform::default(),
        ));
    });

    app.update();

    let mut query = app.world_mut().query::<(&Sun, &DirectionalLight)>();
    let (_, light) = query
        .single(app.world())
        .expect("a managed sun light should exist");
    assert!(light.illuminance >= 0.0);
}

#[test]
fn plugin_does_not_require_auto_spawned_lights() {
    let mut app = App::new();
    let config = DayNightConfig {
        managed_lights: ManagedLightConfig { auto_spawn: false },
        ..default()
    };
    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));
    app.update();
    app.update();

    let sun_count = {
        let mut query = app.world_mut().query::<&Sun>();
        query.iter(app.world()).count()
    };
    let moon_count = {
        let mut query = app.world_mut().query::<&Moon>();
        query.iter(app.world()).count()
    };
    assert_eq!(sun_count, 0);
    assert_eq!(moon_count, 0);
}

#[test]
fn time_reactive_inserts_and_removes_marker() {
    let mut app = App::new();
    let config = DayNightConfig {
        initial_time: 20.0, // night
        paused: true,
        ..default()
    };
    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));
    let entity = app
        .world_mut()
        .spawn((Name::new("Lamp"), crate::TimeReactive::night_active()))
        .id();

    // First update: activates runtime + time reactive system runs
    app.update();
    // Second update: commands from update_time_reactive are applied
    app.update();

    assert!(
        app.world().entity(entity).contains::<crate::TimeActive>(),
        "entity should be TimeActive at 20:00 with night_active window (19–6)"
    );

    // Scrub to midday — should deactivate
    app.world_mut()
        .resource_mut::<DayNightConfig>()
        .queue_scrub(12.0);
    app.update();
    app.update();

    assert!(
        !app.world().entity(entity).contains::<crate::TimeActive>(),
        "entity should NOT be TimeActive at 12:00 with night_active window"
    );
}
