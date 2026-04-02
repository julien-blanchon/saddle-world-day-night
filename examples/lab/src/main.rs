#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use bevy::{
    camera::Exposure, core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom,
    prelude::*,
};
use saddle_world_day_night::{
    CelestialSettings, DayNightCamera, DayNightConfig, DayNightDiagnostics, DayNightLighting,
    DayNightPlugin, DayNightSystems, DayPhase, DuskStarted, Moon, NightStarted, Sun, TimeOfDay,
};

#[derive(Component)]
struct LabSpinner {
    axis: Vec3,
    speed: f32,
}

#[derive(Component)]
struct LabOverlay;

#[derive(Resource, Default, Debug, Clone)]
pub struct PhaseLog {
    pub entries: Vec<DayPhase>,
    pub dawn_count: u32,
    pub day_count: u32,
    pub dusk_count: u32,
    pub night_count: u32,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PerformanceSnapshot {
    pub sun_writes: u64,
    pub moon_writes: u64,
    pub ambient_writes: u64,
    pub fog_writes: u64,
    pub exposure_writes: u64,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(PhaseLog::default());
    app.insert_resource(PerformanceSnapshot::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Day Night Lab".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::E2EPlugin);

    app.add_plugins(DayNightPlugin::default().with_config(lab_config()));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_props,
            update_overlay.after(DayNightSystems::ApplyLighting),
            record_dawn.after(DayNightSystems::DetectPhaseTransitions),
            record_day.after(DayNightSystems::DetectPhaseTransitions),
            record_dusk.after(DayNightSystems::DetectPhaseTransitions),
            record_night.after(DayNightSystems::DetectPhaseTransitions),
        ),
    );
    app.run();
}

fn lab_config() -> DayNightConfig {
    DayNightConfig {
        initial_time: 4.5,
        seconds_per_hour: 0.5,
        celestial: CelestialSettings {
            lunar_phase_offset: 0.5,
            ..default()
        },
        managed_lights: saddle_world_day_night::ManagedLightConfig { auto_spawn: false },
        ..default()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Lab Sun"),
        Sun,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Lab Moon"),
        Moon,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Lab Camera"),
        Camera3d::default(),
        DayNightCamera {
            ensure_atmosphere: true,
            ..default()
        },
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        Transform::from_xyz(-10.0, 6.2, -12.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Lab Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.21, 0.24),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    let props = [
        (
            Vec3::new(-10.0, 1.5, -8.0),
            Vec3::new(1.8, 3.0, 1.8),
            Color::srgb(0.23, 0.40, 0.72),
        ),
        (
            Vec3::new(-4.0, 2.1, -2.0),
            Vec3::new(2.0, 4.2, 2.0),
            Color::srgb(0.76, 0.36, 0.24),
        ),
        (
            Vec3::new(2.5, 1.1, 3.0),
            Vec3::new(1.6, 2.2, 1.6),
            Color::srgb(0.26, 0.68, 0.54),
        ),
        (
            Vec3::new(8.0, 2.5, -4.0),
            Vec3::new(2.4, 5.0, 2.4),
            Color::srgb(0.74, 0.66, 0.20),
        ),
        (
            Vec3::new(4.0, 0.8, 9.0),
            Vec3::new(6.0, 1.6, 3.0),
            Color::srgb(0.34, 0.26, 0.20),
        ),
        (
            Vec3::new(-2.0, 0.9, 10.5),
            Vec3::new(3.2, 1.8, 3.2),
            Color::srgb(0.48, 0.30, 0.22),
        ),
    ];

    for (index, (translation, scale, color)) in props.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Lab Prop {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.04,
                perceptual_roughness: 0.38,
                ..default()
            })),
            Transform::from_translation(translation),
            LabSpinner {
                axis: Vec3::new(0.18 + index as f32 * 0.06, 1.0, 0.14).normalize(),
                speed: 0.10 + index as f32 * 0.03,
            },
        ));
    }

    commands.spawn((
        Name::new("Lab Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(460.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.78)),
        Text::default(),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn animate_props(time: Res<Time>, mut query: Query<(&LabSpinner, &mut Transform)>) {
    for (spinner, mut transform) in &mut query {
        transform.rotate(Quat::from_axis_angle(
            spinner.axis,
            spinner.speed * time.delta_secs(),
        ));
    }
}

fn update_overlay(
    time_of_day: Res<TimeOfDay>,
    lighting: Res<DayNightLighting>,
    diagnostics: Res<DayNightDiagnostics>,
    phase_log: Res<PhaseLog>,
    mut overlay: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    text.0 = format!(
        "Day Night Lab\nTime {:05.2}  Day {}\nPhase {:?}\nSun lux {:>8.1}  Moon lux {:>6.3}\nAmbient {:>5.2}  Fog vis {:>6.1}\nExposure EV100 {:>4.2}  Stars {:>4.2}\nCounts Dn/Day/Dk/Nt = {}/{}/{}/{}\nWrites sun {} moon {} ambient {} fog {} exposure {}",
        time_of_day.hour,
        time_of_day.elapsed_days,
        diagnostics.current_phase,
        lighting.sun_illuminance_lux,
        lighting.moon_illuminance_lux,
        lighting.ambient_brightness,
        lighting.fog_visibility,
        lighting.suggested_exposure_ev100,
        lighting.star_visibility,
        phase_log.dawn_count,
        phase_log.day_count,
        phase_log.dusk_count,
        phase_log.night_count,
        diagnostics.sun_writes,
        diagnostics.moon_writes,
        diagnostics.ambient_writes,
        diagnostics.fog_writes,
        diagnostics.exposure_writes,
    );
}

fn record_dawn(
    mut phase_log: ResMut<PhaseLog>,
    mut messages: MessageReader<saddle_world_day_night::DawnStarted>,
) {
    let count = messages.read().count() as u32;
    if count > 0 {
        phase_log.dawn_count += count;
        phase_log
            .entries
            .extend(std::iter::repeat_n(DayPhase::Dawn, count as usize));
    }
}

fn record_day(mut phase_log: ResMut<PhaseLog>, mut messages: MessageReader<saddle_world_day_night::DayStarted>) {
    let count = messages.read().count() as u32;
    if count > 0 {
        phase_log.day_count += count;
        phase_log
            .entries
            .extend(std::iter::repeat_n(DayPhase::Day, count as usize));
    }
}

fn record_dusk(mut phase_log: ResMut<PhaseLog>, mut messages: MessageReader<DuskStarted>) {
    let count = messages.read().count() as u32;
    if count > 0 {
        phase_log.dusk_count += count;
        phase_log
            .entries
            .extend(std::iter::repeat_n(DayPhase::Dusk, count as usize));
    }
}

fn record_night(mut phase_log: ResMut<PhaseLog>, mut messages: MessageReader<NightStarted>) {
    let count = messages.read().count() as u32;
    if count > 0 {
        phase_log.night_count += count;
        phase_log
            .entries
            .extend(std::iter::repeat_n(DayPhase::Night, count as usize));
    }
}
