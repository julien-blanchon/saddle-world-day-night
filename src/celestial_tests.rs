use bevy::prelude::*;

use super::{
    CelestialModel, CelestialSettings, SeasonSettings, solar_daylight_window, solve_celestial_state,
};
use crate::{DayPhaseBoundaries, TimeOfDay};

#[test]
fn noon_and_midnight_have_opposite_solar_elevation_signs() {
    let settings = CelestialSettings::default();
    let boundaries = DayPhaseBoundaries::default();

    let noon = solve_celestial_state(TimeOfDay::new(12.0), &boundaries, &settings, 14.0);
    let midnight = solve_celestial_state(TimeOfDay::new(0.0), &boundaries, &settings, 9.0);

    assert!(noon.sun_elevation_degrees > 60.0);
    assert!(midnight.sun_elevation_degrees < -60.0);
    assert!(noon.sun_direction.y < 0.0);
    assert!(midnight.sun_direction.y > 0.0);
}

#[test]
fn latitude_changes_noon_height_and_day_length() {
    let equator = CelestialSettings {
        model: CelestialModel::LatitudeAware {
            latitude_degrees: 0.0,
            season: SeasonSettings {
                season_progress: 0.25,
                ..default()
            },
        },
        ..default()
    };
    let northern = CelestialSettings {
        model: CelestialModel::LatitudeAware {
            latitude_degrees: 60.0,
            season: SeasonSettings {
                season_progress: 0.25,
                ..default()
            },
        },
        ..default()
    };

    let boundaries = DayPhaseBoundaries::default();
    let equator_state = solve_celestial_state(TimeOfDay::new(12.0), &boundaries, &equator, 14.0);
    let northern_state = solve_celestial_state(TimeOfDay::new(12.0), &boundaries, &northern, 14.0);
    let (equator_rise, equator_set) = solar_daylight_window(&equator);
    let (northern_rise, northern_set) = solar_daylight_window(&northern);

    assert!(equator_state.sun_elevation_degrees > northern_state.sun_elevation_degrees);
    assert!(northern_set - northern_rise > equator_set - equator_rise);
}

#[test]
fn moon_offset_and_phase_cycle_are_configurable() {
    let settings = CelestialSettings {
        moon_hour_offset: 6.0,
        lunar_period_days: 10.0,
        ..default()
    };
    let boundaries = DayPhaseBoundaries::default();

    let first = solve_celestial_state(TimeOfDay::with_days(12.0, 0), &boundaries, &settings, 14.0);
    let later = solve_celestial_state(TimeOfDay::with_days(12.0, 6), &boundaries, &settings, 14.0);

    assert!((first.moon_azimuth_degrees - first.sun_azimuth_degrees).abs() > 40.0);
    assert_ne!(first.moon_phase, later.moon_phase);
}

#[test]
fn wrap_near_midnight_is_continuous_and_finite() {
    let settings = CelestialSettings {
        model: CelestialModel::SimpleArc {
            peak_elevation_degrees: 70.0,
        },
        ..default()
    };
    let boundaries = DayPhaseBoundaries::default();
    let before = solve_celestial_state(TimeOfDay::new(23.9), &boundaries, &settings, 9.0);
    let after = solve_celestial_state(TimeOfDay::new(0.1), &boundaries, &settings, 9.0);

    assert!(before.sun_direction.is_finite());
    assert!(after.sun_direction.is_finite());
    assert!(before.sun_direction.dot(after.sun_direction) > 0.9);
}
