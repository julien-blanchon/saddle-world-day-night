use super::{
    DayNightLighting, LightingProfile, WeatherModulation, kelvin_to_color, resolve_lighting,
    smooth_lighting,
};
use crate::{
    CelestialSettings, DayPhaseBoundaries, ShadowConfig, SmoothingConfig, TimeOfDay, TimeStepMode,
    solve_celestial_state,
};

fn sample(hour: f32) -> super::DayNightLighting {
    let settings = CelestialSettings::default();
    let boundaries = DayPhaseBoundaries::default();
    let celestial = solve_celestial_state(TimeOfDay::new(hour), &boundaries, &settings, 12.0);
    resolve_lighting(
        TimeOfDay::new(hour),
        &celestial,
        &LightingProfile::realistic_outdoor(),
        &WeatherModulation::default(),
        &ShadowConfig::default(),
    )
}

#[test]
fn noon_is_brighter_than_dawn_and_night() {
    let dawn = sample(6.5);
    let noon = sample(12.0);
    let night = sample(0.0);

    assert!(noon.sun_illuminance_lux > dawn.sun_illuminance_lux);
    assert!(dawn.sun_illuminance_lux > night.sun_illuminance_lux);
    assert!(night.sun_illuminance_lux < 1.0);
}

#[test]
fn realistic_defaults_keep_night_readable_for_showcases() {
    let night = sample(0.0);

    assert!(night.ambient_brightness >= 50.0);
    assert!(night.suggested_exposure_ev100 <= 4.0);
}

#[test]
fn moonlit_preset_keeps_night_biased_scenes_visible() {
    let settings = CelestialSettings {
        lunar_phase_offset: 0.5,
        ..Default::default()
    };
    let boundaries = DayPhaseBoundaries::default();
    let celestial = solve_celestial_state(TimeOfDay::new(0.0), &boundaries, &settings, 2.0);
    let lighting = resolve_lighting(
        TimeOfDay::new(0.0),
        &celestial,
        &LightingProfile::moonlit_night(),
        &WeatherModulation::default(),
        &ShadowConfig::default(),
    );

    assert!(lighting.ambient_brightness >= 60.0);
    assert!(lighting.moon_illuminance_lux >= 1.0);
    assert!(lighting.suggested_exposure_ev100 <= 3.0);
}

#[test]
fn ambient_and_fog_outputs_stay_non_negative() {
    for hour in [0.0, 5.5, 7.0, 12.0, 18.0, 23.5] {
        let lighting = sample(hour);
        assert!(lighting.ambient_brightness >= 0.0);
        assert!(lighting.fog_density >= 0.0);
        assert!(lighting.fog_visibility >= 0.0);
        assert!(lighting.suggested_exposure_ev100.is_finite());
    }
}

#[test]
fn kelvin_conversion_tracks_warm_and_cool_ranges() {
    let warm = kelvin_to_color(2_000.0).to_linear();
    let neutral = kelvin_to_color(6_500.0).to_linear();
    let cool = kelvin_to_color(10_000.0).to_linear();

    assert!(warm.red > warm.blue);
    assert!((neutral.red - neutral.blue).abs() < 0.2);
    assert!(cool.blue > cool.red);
}

#[test]
fn weather_modulation_dims_direct_light_and_thickens_fog() {
    let settings = CelestialSettings::default();
    let boundaries = DayPhaseBoundaries::default();
    let celestial = solve_celestial_state(TimeOfDay::new(12.0), &boundaries, &settings, 14.0);
    let profile = LightingProfile::realistic_outdoor();

    let clear = resolve_lighting(
        TimeOfDay::new(12.0),
        &celestial,
        &profile,
        &WeatherModulation::default(),
        &ShadowConfig::default(),
    );
    let storm = resolve_lighting(
        TimeOfDay::new(12.0),
        &celestial,
        &profile,
        &WeatherModulation {
            cloud_cover: 0.9,
            haze: 0.8,
            precipitation_dimming: 0.7,
        },
        &ShadowConfig::default(),
    );

    assert!(storm.sun_illuminance_lux < clear.sun_illuminance_lux);
    assert!(storm.fog_density > clear.fog_density);
    assert!(storm.fog_visibility < clear.fog_visibility);
}

#[test]
fn smoothing_does_not_delay_shadow_flag_changes() {
    let current = DayNightLighting {
        sun_shadows_enabled: false,
        moon_shadows_enabled: false,
        ..Default::default()
    };
    let target = DayNightLighting {
        sun_shadows_enabled: true,
        moon_shadows_enabled: true,
        ..Default::default()
    };

    let smoothed = smooth_lighting(
        &current,
        &target,
        &SmoothingConfig {
            continuous_seconds: 0.18,
            ..Default::default()
        },
        TimeStepMode::Continuous,
        1.0 / 60.0,
    );

    assert!(smoothed.sun_shadows_enabled);
    assert!(smoothed.moon_shadows_enabled);
}

#[test]
fn idle_steps_snap_directly_to_the_target() {
    let current = DayNightLighting {
        ambient_brightness: 12.0,
        fog_visibility: 120.0,
        ..Default::default()
    };
    let target = DayNightLighting {
        ambient_brightness: 42.0,
        fog_visibility: 650.0,
        ..Default::default()
    };

    let resolved = smooth_lighting(
        &current,
        &target,
        &SmoothingConfig {
            continuous_seconds: 0.18,
            jump_seconds: 1.0,
            smooth_scrubs: true,
        },
        TimeStepMode::Idle,
        1.0 / 60.0,
    );

    assert_eq!(resolved.ambient_brightness, target.ambient_brightness);
    assert_eq!(resolved.fog_visibility, target.fog_visibility);
}
