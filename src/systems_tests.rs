use bevy::pbr::{Atmosphere, DistanceFog, ScatteringMedium};
use bevy::prelude::*;

use crate::{
    DawnStarted, DayNightCamera, DayNightConfig, DayNightPlugin, DayNightSystems, DayStarted,
    DuskStarted, GlobalAmbientConfig, ManagedLightConfig, Moon, NightStarted, Sun,
};

#[derive(Resource, Default)]
struct PhaseCounts {
    dawn: u32,
    day: u32,
    dusk: u32,
    night: u32,
}

#[derive(Resource, Default)]
struct FogChangeCounts {
    distance_fog_changes: u32,
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

fn count_distance_fog_changes(
    mut counts: ResMut<FogChangeCounts>,
    fogs: Query<Ref<DistanceFog>, With<DayNightCamera>>,
) {
    for fog in &fogs {
        if fog.is_changed() {
            counts.distance_fog_changes += 1;
        }
    }
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

#[test]
fn plugin_can_leave_global_ambient_to_the_consumer() {
    let mut app = App::new();
    let ambient = GlobalAmbientLight {
        color: Color::srgb(0.12, 0.18, 0.24),
        brightness: 7.5,
        ..default()
    };
    let config = DayNightConfig {
        global_ambient: GlobalAmbientConfig { apply: false },
        ..default()
    };

    app.insert_resource(ambient.clone());
    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));

    app.update();

    let current = app.world().resource::<GlobalAmbientLight>();
    assert_eq!(current.color, ambient.color);
    assert_eq!(current.brightness, ambient.brightness);
}

#[test]
fn paused_runtime_stops_rewriting_distance_fog_once_settled() {
    let mut app = App::new();
    let config = DayNightConfig::default().fixed_time(12.0);

    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));
    app.insert_resource(FogChangeCounts::default());
    app.add_systems(
        Update,
        count_distance_fog_changes.after(DayNightSystems::ApplyLighting),
    );
    let _camera = app
        .world_mut()
        .spawn((
            Name::new("Fog Camera"),
            Camera3d::default(),
            DayNightCamera::default(),
            Transform::default(),
        ))
        .id();

    app.update();
    app.update();
    for _ in 0..180 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<FogChangeCounts>()
        .distance_fog_changes = 0;

    app.update();

    let counts = app.world().resource::<FogChangeCounts>();
    assert_eq!(
        counts.distance_fog_changes, 0,
        "DistanceFog should stay unchanged on stable paused frames"
    );
}

#[test]
fn atmosphere_density_changes_refresh_existing_camera_medium() {
    let mut app = App::new();
    let config = DayNightConfig::default().fixed_time(12.0);

    app.insert_resource(Assets::<ScatteringMedium>::default());
    app.add_plugins((
        MinimalPlugins,
        DayNightPlugin::default().with_config(config),
    ));

    let camera = app
        .world_mut()
        .spawn((
            Name::new("Atmosphere Camera"),
            Camera3d::default(),
            DayNightCamera {
                ensure_atmosphere: true,
                ..default()
            },
            Transform::default(),
        ))
        .id();

    app.update();
    app.update();

    let initial_medium = app
        .world()
        .entity(camera)
        .get::<Atmosphere>()
        .expect("camera should receive an atmosphere component")
        .medium
        .clone();

    app.world_mut()
        .resource_mut::<DayNightConfig>()
        .atmosphere
        .density_multiplier = 1.8;
    app.update();
    app.update();

    let updated_medium = app
        .world()
        .entity(camera)
        .get::<Atmosphere>()
        .expect("camera atmosphere should remain present")
        .medium
        .clone();

    assert_ne!(
        initial_medium, updated_medium,
        "changing density_multiplier should refresh the managed atmosphere medium"
    );
}
