use saddle_world_day_night_example_support as support;

use bevy::{
    camera::Exposure, core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom,
    prelude::*,
};
use height_fog::HeightFog;
use saddle_world_day_night::{
    DayNightCamera, DayNightConfig, DayNightLighting, DayNightPlugin, DayNightSystems,
    TimeOfDay, WeatherModulation,
};
use saddle_world_sky::{SkyCamera, SkyConfig, SkyPlugin, SkyState};
use saddle_world_weather::{WeatherCamera, WeatherConfig, WeatherPlugin, WeatherProfile, WeatherRuntime};

#[derive(Resource)]
struct AtmosphereCycle {
    timer: Timer,
    index: usize,
}

fn main() {
    let day_night_config = DayNightConfig {
        initial_time: 6.0,
        seconds_per_hour: 1.4,
        paused: true,
        ..default()
    };
    let sky_config = SkyConfig {
        time_of_day: saddle_world_sky::TimeOfDay {
            hours: 6.0,
            day_length_seconds: 33.6,
            speed: 1.0,
            paused: false,
            elapsed_days: 0,
            pending_override: None,
        },
        ..default()
    };
    let weather_config = WeatherConfig {
        initial_profile: WeatherProfile::clear(),
        quality: saddle_world_weather::WeatherQuality::High,
        seed: 17,
        default_transition_duration_secs: 2.8,
        ..default()
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(AtmosphereCycle {
        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        index: 0,
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night atmosphere".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &day_night_config);
    app.add_plugins(DayNightPlugin::default().with_config(day_night_config));
    app.add_plugins(SkyPlugin::default().with_config(sky_config));
    app.add_plugins(WeatherPlugin::default().with_config(weather_config));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::spin_showcase,
            cycle_weather_profiles,
            sync_day_night_time.after(saddle_world_sky::SkySystems::AdvanceTime),
            sync_weather_modulation.before(DayNightSystems::ResolveLighting),
            update_overlay
                .after(saddle_world_sky::SkySystems::EmitEvents)
                .after(saddle_world_weather::WeatherSystems::Diagnostics)
                .after(DayNightSystems::ApplyLighting),
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = support::spawn_outdoor_showcase(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        DayNightCamera {
            apply_distance_fog: false,
            apply_volumetric_fog: false,
            apply_exposure: false,
            apply_environment_map_light: false,
            ensure_atmosphere: false,
            ..default()
        },
        true,
    );
    commands.entity(camera).insert((
        SkyCamera::default(),
        WeatherCamera {
            receive_screen_fx: false,
            ..default()
        },
        HeightFog::dense_ground_fog(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
    ));
}

fn cycle_weather_profiles(
    time: Res<Time>,
    mut cycle: ResMut<AtmosphereCycle>,
    mut config: ResMut<WeatherConfig>,
) {
    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    let profiles = [
        WeatherProfile::clear(),
        WeatherProfile::foggy(),
        WeatherProfile::rain(),
        WeatherProfile::storm(),
        WeatherProfile::snow(),
    ];
    config.queue_transition(profiles[cycle.index % profiles.len()].clone(), 2.2);
    cycle.index += 1;
}

fn sync_day_night_time(
    sky_time: Res<saddle_world_sky::TimeOfDay>,
    mut day_time: ResMut<TimeOfDay>,
) {
    day_time.set_hour(sky_time.hours);
    day_time.elapsed_days = sky_time.elapsed_days.max(0) as u32;
}

fn sync_weather_modulation(
    runtime: Res<WeatherRuntime>,
    mut weather: ResMut<WeatherModulation>,
) {
    weather.cloud_cover = runtime
        .factors
        .rain_factor
        .max(runtime.factors.snow_factor * 0.9)
        .max(runtime.factors.storm_factor * 0.75)
        .clamp(0.0, 1.0);
    weather.haze = runtime.factors.fog_factor.clamp(0.0, 1.0);
    weather.precipitation_dimming = runtime
        .factors
        .rain_factor
        .max(runtime.factors.snow_factor)
        .max(runtime.factors.storm_factor * 0.85)
        .clamp(0.0, 1.0);
}

fn update_overlay(
    time_of_day: Res<TimeOfDay>,
    lighting: Res<DayNightLighting>,
    sky: Res<SkyState>,
    weather: Res<WeatherRuntime>,
    mut overlay: Query<&mut Text, With<support::ShowcaseOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    text.0 = format!(
        "Atmosphere Stack\nTime {:05.2}  Weather {}\nSun {:>8.0} lux  Fog vis {:>6.1}\nSky band {:?}  Clouds {:>4.2}  Stars {:>4.2}\nWeather rain {:>4.2}  snow {:>4.2}  wetness {:>4.2}\nHeight fog enabled via post-process camera stack",
        time_of_day.hour,
        weather.active_profile.label.as_deref().unwrap_or("Unnamed"),
        lighting.sun_illuminance_lux,
        lighting.fog_visibility,
        sky.current_band,
        sky.cloud_cover_factor,
        sky.star_visibility,
        weather.factors.rain_factor,
        weather.factors.snow_factor,
        weather.factors.wetness_factor,
    );
}
