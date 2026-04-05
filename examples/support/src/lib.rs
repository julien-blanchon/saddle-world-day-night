use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_day_night::{
    CelestialState, DayNightCamera, DayNightConfig, DayNightDiagnostics, DayNightLighting, Moon,
    Sun, TimeOfDay, WeatherModulation,
};

#[derive(Component)]
pub struct ShowcaseSpinner {
    pub axis: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct ShowcaseOverlay;

#[derive(Resource, Clone, Default, Pane)]
#[pane(title = "Day Night Controls", position = "top-right")]
pub struct DayNightDemoPane {
    #[pane]
    pub paused: bool,
    #[pane(slider, min = 0.0, max = 24.0, step = 0.1)]
    pub time_hours: f32,
    #[pane(slider, min = 0.25, max = 8.0, step = 0.05)]
    pub seconds_per_hour: f32,
    #[pane(slider, min = 0.0, max = 8.0, step = 0.05)]
    pub time_scale: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    pub cloud_cover: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    pub haze: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    pub precipitation_dimming: f32,
    #[pane(monitor)]
    pub sun_lux: f32,
    #[pane(monitor)]
    pub fog_visibility: f32,
    #[pane(monitor)]
    pub star_visibility: f32,
    #[pane(monitor)]
    pub phase: String,
}

impl DayNightDemoPane {
    pub fn from_config(config: &DayNightConfig) -> Self {
        Self {
            paused: config.paused,
            time_hours: config.initial_time,
            seconds_per_hour: config.seconds_per_hour,
            time_scale: config.time_scale,
            cloud_cover: 0.0,
            haze: 0.0,
            precipitation_dimming: 0.0,
            sun_lux: 0.0,
            fog_visibility: 0.0,
            star_visibility: 0.0,
            phase: String::new(),
        }
    }
}

/// Tracks the last values the monitor wrote so we can detect user-initiated changes.
#[derive(Resource, Default)]
struct PaneSyncState {
    last_monitor_time: f32,
    last_monitor_seconds_per_hour: f32,
    last_monitor_time_scale: f32,
    last_monitor_cloud_cover: f32,
    last_monitor_haze: f32,
    last_monitor_precipitation: f32,
}

pub fn install_demo_pane(app: &mut App, config: &DayNightConfig) {
    let pane = DayNightDemoPane::from_config(config);
    let sync_state = PaneSyncState {
        last_monitor_time: pane.time_hours,
        last_monitor_seconds_per_hour: pane.seconds_per_hour,
        last_monitor_time_scale: pane.time_scale,
        last_monitor_cloud_cover: pane.cloud_cover,
        last_monitor_haze: pane.haze,
        last_monitor_precipitation: pane.precipitation_dimming,
    };
    app.insert_resource(pane);
    app.insert_resource(sync_state);
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<DayNightDemoPane>();
    app.add_systems(
        Update,
        (
            sync_demo_pane.before(saddle_world_day_night::DayNightSystems::AdvanceTime),
            sync_demo_monitors.after(saddle_world_day_night::DayNightSystems::ResolveLighting),
        ),
    );
}

pub fn spawn_outdoor_showcase(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    camera: DayNightCamera,
    with_overlay: bool,
) -> Entity {
    commands.spawn((
        Name::new("Showcase Sun"),
        Sun,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Showcase Moon"),
        Moon,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default(),
    ));
    let camera_entity = commands
        .spawn((
            Name::new("Showcase Camera"),
            Camera3d::default(),
            camera,
            Transform::from_xyz(-9.0, 6.0, -12.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        ))
        .id();

    commands.spawn((
        Name::new("Showcase Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.21, 0.24),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    let palette = [
        (
            Vec3::new(-8.0, 1.3, -8.0),
            Vec3::new(1.4, 2.6, 1.4),
            Color::srgb(0.27, 0.42, 0.68),
        ),
        (
            Vec3::new(-2.5, 1.7, -3.0),
            Vec3::new(2.4, 3.4, 2.0),
            Color::srgb(0.74, 0.39, 0.28),
        ),
        (
            Vec3::new(3.2, 1.1, 2.2),
            Vec3::new(1.8, 2.2, 1.8),
            Color::srgb(0.28, 0.64, 0.46),
        ),
        (
            Vec3::new(9.0, 2.1, -1.4),
            Vec3::new(2.2, 4.2, 2.2),
            Color::srgb(0.78, 0.68, 0.24),
        ),
        (
            Vec3::new(0.0, 0.7, 8.4),
            Vec3::new(5.0, 1.4, 2.8),
            Color::srgb(0.38, 0.28, 0.22),
        ),
    ];

    for (index, (translation, scale, color)) in palette.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Showcase Prop {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.05,
                perceptual_roughness: 0.42,
                ..default()
            })),
            Transform::from_translation(translation),
            ShowcaseSpinner {
                axis: Vec3::new(0.2 + index as f32 * 0.07, 1.0, 0.18).normalize(),
                speed: 0.08 + index as f32 * 0.03,
            },
        ));
    }

    if with_overlay {
        commands.spawn((
            Name::new("Showcase Overlay"),
            ShowcaseOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(420.0),
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.72)),
            Text::default(),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }

    camera_entity
}

/// Spawns an instructions text overlay at the bottom-left of the screen.
pub fn spawn_instructions(commands: &mut Commands, text: &str) {
    commands.spawn((
        Name::new("Instructions"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(440.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.60)),
        Text::new(text.to_string()),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.85, 0.9)),
    ));
}

pub fn spin_showcase(time: Res<Time>, mut query: Query<(&ShowcaseSpinner, &mut Transform)>) {
    for (spinner, mut transform) in &mut query {
        transform.rotate(Quat::from_axis_angle(
            spinner.axis,
            spinner.speed * time.delta_secs(),
        ));
    }
}

pub fn update_overlay(
    time_of_day: Res<TimeOfDay>,
    celestial: Res<CelestialState>,
    lighting: Res<DayNightLighting>,
    diagnostics: Res<DayNightDiagnostics>,
    mut overlay: Query<&mut Text, With<ShowcaseOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    text.0 = format!(
        "Time {:05.2}  Day {}\nPhase {:?}\nSun elevation {:>6.2}°  Moon elevation {:>6.2}°\nSun lux {:>8.1}  Moon lux {:>6.3}\nAmbient {:>5.2}  Fog vis {:>6.1}\nExposure EV100 {:>4.2}  Stars {:>4.2}\nPhase messages {}",
        time_of_day.hour,
        time_of_day.elapsed_days,
        celestial.phase,
        celestial.sun_elevation_degrees,
        celestial.moon_elevation_degrees,
        lighting.sun_illuminance_lux,
        lighting.moon_illuminance_lux,
        lighting.ambient_brightness,
        lighting.fog_visibility,
        lighting.suggested_exposure_ev100,
        lighting.star_visibility,
        diagnostics.phase_message_count,
    );
}

/// Detects user changes to pane sliders by comparing against the last monitor-written values.
/// Only applies changes that differ from what the monitor last wrote (i.e. user-initiated edits).
fn sync_demo_pane(
    pane: Res<DayNightDemoPane>,
    sync: Res<PaneSyncState>,
    mut config: ResMut<DayNightConfig>,
    mut weather: ResMut<WeatherModulation>,
) {
    // Pause is never written by the monitor, so always apply directly.
    if config.paused != pane.paused {
        config.paused = pane.paused;
    }

    // Detect user-initiated time scrub: the pane value differs from what the monitor last wrote.
    let user_changed_time = (pane.time_hours - sync.last_monitor_time).abs() > 0.05;
    if user_changed_time {
        config.queue_scrub(pane.time_hours);
    }

    // seconds_per_hour: detect user change vs monitor echo
    let user_changed_sph =
        (pane.seconds_per_hour - sync.last_monitor_seconds_per_hour).abs() > 0.01;
    if user_changed_sph {
        let desired = pane.seconds_per_hour.max(0.01);
        if (config.seconds_per_hour - desired).abs() > f32::EPSILON {
            config.seconds_per_hour = desired;
        }
    }

    // time_scale: detect user change vs monitor echo
    let user_changed_ts = (pane.time_scale - sync.last_monitor_time_scale).abs() > 0.01;
    if user_changed_ts {
        let desired = pane.time_scale.max(0.0);
        if (config.time_scale - desired).abs() > f32::EPSILON {
            config.time_scale = desired;
        }
    }

    // Weather controls: detect user change vs monitor echo
    let desired_cloud_cover = pane.cloud_cover.clamp(0.0, 1.0);
    let desired_haze = pane.haze.clamp(0.0, 1.0);
    let desired_precipitation = pane.precipitation_dimming.clamp(0.0, 1.0);

    if (pane.cloud_cover - sync.last_monitor_cloud_cover).abs() > 0.005 {
        weather.cloud_cover = desired_cloud_cover;
    }
    if (pane.haze - sync.last_monitor_haze).abs() > 0.005 {
        weather.haze = desired_haze;
    }
    if (pane.precipitation_dimming - sync.last_monitor_precipitation).abs() > 0.005 {
        weather.precipitation_dimming = desired_precipitation;
    }
}

/// Writes current simulation state back to the pane for display, and records what was written
/// so that `sync_demo_pane` can distinguish user edits from monitor echo-back.
fn sync_demo_monitors(
    time_of_day: Res<TimeOfDay>,
    celestial: Res<CelestialState>,
    lighting: Res<DayNightLighting>,
    config: Res<DayNightConfig>,
    weather: Res<WeatherModulation>,
    mut pane: ResMut<DayNightDemoPane>,
    mut sync: ResMut<PaneSyncState>,
) {
    // Update time slider to reflect current simulation time
    pane.time_hours = time_of_day.hour;
    sync.last_monitor_time = time_of_day.hour;

    // Update config-driven sliders
    pane.seconds_per_hour = config.seconds_per_hour;
    sync.last_monitor_seconds_per_hour = config.seconds_per_hour;

    pane.time_scale = config.time_scale;
    sync.last_monitor_time_scale = config.time_scale;

    // Update weather sliders from resource (round-trip)
    pane.cloud_cover = weather.cloud_cover;
    sync.last_monitor_cloud_cover = weather.cloud_cover;

    pane.haze = weather.haze;
    sync.last_monitor_haze = weather.haze;

    pane.precipitation_dimming = weather.precipitation_dimming;
    sync.last_monitor_precipitation = weather.precipitation_dimming;

    // Update monitors
    pane.sun_lux = lighting.sun_illuminance_lux;
    pane.fog_visibility = lighting.fog_visibility;
    pane.star_visibility = lighting.star_visibility;
    pane.phase = format!("{:?}", celestial.phase);
    pane.paused = config.paused;
}
