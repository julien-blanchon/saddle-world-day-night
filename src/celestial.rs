use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::{DayPhase, DayPhaseBoundaries, TimeOfDay};

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct SeasonSettings {
    pub axial_tilt_degrees: f32,
    pub season_progress: f32,
}

impl Default for SeasonSettings {
    fn default() -> Self {
        Self {
            axial_tilt_degrees: 23.4,
            season_progress: 0.0,
        }
    }
}

impl SeasonSettings {
    pub fn declination_degrees(&self) -> f32 {
        self.axial_tilt_degrees * (self.season_progress.fract() * TAU).sin()
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub enum CelestialModel {
    SimpleArc {
        peak_elevation_degrees: f32,
    },
    LatitudeAware {
        latitude_degrees: f32,
        season: SeasonSettings,
    },
}

impl Default for CelestialModel {
    fn default() -> Self {
        Self::SimpleArc {
            peak_elevation_degrees: 72.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct CelestialSettings {
    pub model: CelestialModel,
    pub azimuth_offset_degrees: f32,
    pub moon_hour_offset: f32,
    pub moon_elevation_offset_degrees: f32,
    pub lunar_period_days: f32,
    pub lunar_phase_offset: f32,
}

impl Default for CelestialSettings {
    fn default() -> Self {
        Self {
            model: CelestialModel::default(),
            azimuth_offset_degrees: 0.0,
            moon_hour_offset: 12.0,
            moon_elevation_offset_degrees: 8.0,
            lunar_period_days: 29.53,
            lunar_phase_offset: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    pub fn from_fraction(phase_fraction: f32) -> Self {
        match phase_fraction.rem_euclid(1.0) {
            value if !(0.0625..0.9375).contains(&value) => MoonPhase::New,
            value if value < 0.1875 => MoonPhase::WaxingCrescent,
            value if value < 0.3125 => MoonPhase::FirstQuarter,
            value if value < 0.4375 => MoonPhase::WaxingGibbous,
            value if value < 0.5625 => MoonPhase::Full,
            value if value < 0.6875 => MoonPhase::WaningGibbous,
            value if value < 0.8125 => MoonPhase::LastQuarter,
            _ => MoonPhase::WaningCrescent,
        }
    }

    pub fn brightness_factor(self) -> f32 {
        match self {
            MoonPhase::New => 0.08,
            MoonPhase::WaxingCrescent | MoonPhase::WaningCrescent => 0.35,
            MoonPhase::FirstQuarter | MoonPhase::LastQuarter => 0.58,
            MoonPhase::WaxingGibbous | MoonPhase::WaningGibbous => 0.82,
            MoonPhase::Full => 1.0,
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default, Clone)]
pub struct CelestialState {
    pub sun_direction: Vec3,
    pub moon_direction: Vec3,
    pub sun_elevation_degrees: f32,
    pub moon_elevation_degrees: f32,
    pub sun_azimuth_degrees: f32,
    pub moon_azimuth_degrees: f32,
    pub sunrise_hour: f32,
    pub sunset_hour: f32,
    pub moon_phase: MoonPhase,
    pub moon_phase_fraction: f32,
    pub star_visibility: f32,
    pub daylight_factor: f32,
    pub night_factor: f32,
    pub golden_hour_factor: f32,
    pub twilight_factor: f32,
    pub phase: DayPhase,
    pub suggested_exposure_ev100: f32,
}

impl Default for CelestialState {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::NEG_Y,
            moon_direction: Vec3::Y,
            sun_elevation_degrees: 72.0,
            moon_elevation_degrees: -56.0,
            sun_azimuth_degrees: 180.0,
            moon_azimuth_degrees: 0.0,
            sunrise_hour: 6.0,
            sunset_hour: 18.0,
            moon_phase: MoonPhase::Full,
            moon_phase_fraction: 0.5,
            star_visibility: 0.0,
            daylight_factor: 1.0,
            night_factor: 0.0,
            golden_hour_factor: 0.0,
            twilight_factor: 0.0,
            phase: DayPhase::Day,
            suggested_exposure_ev100: 14.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SolarSample {
    direction: Vec3,
    elevation_degrees: f32,
    azimuth_degrees: f32,
    sunrise_hour: f32,
    sunset_hour: f32,
}

pub fn solve_celestial_state(
    time_of_day: TimeOfDay,
    phase_boundaries: &DayPhaseBoundaries,
    settings: &CelestialSettings,
    suggested_exposure_ev100: f32,
) -> CelestialState {
    let hour = time_of_day.cyclic_hour();
    let sun = solve_solar_sample(hour, settings);
    let mut moon = solve_solar_sample(hour + settings.moon_hour_offset, settings);
    moon.elevation_degrees += settings.moon_elevation_offset_degrees;
    moon.direction = direction_from_alt_az(moon.elevation_degrees, moon.azimuth_degrees);

    let moon_phase_fraction = ((time_of_day.total_hours() as f32 / 24.0)
        / settings.lunar_period_days.max(0.1)
        + settings.lunar_phase_offset)
        .fract();
    let moon_phase = MoonPhase::from_fraction(moon_phase_fraction);

    let daylight_factor = smoothstep(-6.0, 8.0, sun.elevation_degrees);
    let twilight_factor = smoothstep(-18.0, 0.0, sun.elevation_degrees)
        * (1.0 - smoothstep(0.0, 10.0, sun.elevation_degrees));
    let night_factor = (1.0 - smoothstep(-6.0, 4.0, sun.elevation_degrees)).clamp(0.0, 1.0);
    let star_visibility = smoothstep(-6.0, -18.0, sun.elevation_degrees) * (1.0 - daylight_factor);
    let golden_hour_factor = triangle_peak(sun.elevation_degrees, 5.5, 0.0, 11.0)
        * smoothstep(-1.0, 1.0, sun.elevation_degrees);

    CelestialState {
        sun_direction: sun.direction,
        moon_direction: moon.direction,
        sun_elevation_degrees: sun.elevation_degrees,
        moon_elevation_degrees: moon.elevation_degrees,
        sun_azimuth_degrees: sun.azimuth_degrees,
        moon_azimuth_degrees: moon.azimuth_degrees,
        sunrise_hour: sun.sunrise_hour,
        sunset_hour: sun.sunset_hour,
        moon_phase,
        moon_phase_fraction,
        star_visibility,
        daylight_factor,
        night_factor,
        golden_hour_factor,
        twilight_factor,
        phase: phase_boundaries.phase_at(hour),
        suggested_exposure_ev100,
    }
}

pub fn solar_daylight_window(settings: &CelestialSettings) -> (f32, f32) {
    match &settings.model {
        CelestialModel::SimpleArc { .. } => (6.0, 18.0),
        CelestialModel::LatitudeAware {
            latitude_degrees,
            season,
        } => daylight_window_from_declination(*latitude_degrees, season.declination_degrees()),
    }
}

fn solve_solar_sample(hour: f32, settings: &CelestialSettings) -> SolarSample {
    match &settings.model {
        CelestialModel::SimpleArc {
            peak_elevation_degrees,
        } => solve_simple_arc(
            hour,
            *peak_elevation_degrees,
            settings.azimuth_offset_degrees,
        ),
        CelestialModel::LatitudeAware {
            latitude_degrees,
            season,
        } => solve_latitude_aware(
            hour,
            *latitude_degrees,
            season.declination_degrees(),
            settings.azimuth_offset_degrees,
        ),
    }
}

fn solve_simple_arc(
    hour: f32,
    peak_elevation_degrees: f32,
    azimuth_offset_degrees: f32,
) -> SolarSample {
    let cyclic_hour = hour.rem_euclid(24.0);
    let elevation_degrees = peak_elevation_degrees * ((cyclic_hour - 6.0) / 24.0 * TAU).sin();
    let azimuth_degrees = (cyclic_hour / 24.0 * 360.0 + azimuth_offset_degrees).rem_euclid(360.0);

    SolarSample {
        direction: direction_from_alt_az(elevation_degrees, azimuth_degrees),
        elevation_degrees,
        azimuth_degrees,
        sunrise_hour: 6.0,
        sunset_hour: 18.0,
    }
}

fn solve_latitude_aware(
    hour: f32,
    latitude_degrees: f32,
    declination_degrees: f32,
    azimuth_offset_degrees: f32,
) -> SolarSample {
    let cyclic_hour = hour.rem_euclid(24.0);
    let lat = latitude_degrees.to_radians();
    let declination = declination_degrees.to_radians();
    let hour_angle = (cyclic_hour - 12.0) / 24.0 * TAU;
    let sin_elevation =
        lat.sin() * declination.sin() + lat.cos() * declination.cos() * hour_angle.cos();
    let elevation = sin_elevation.clamp(-1.0, 1.0).asin();
    let elevation_degrees = elevation.to_degrees();

    let cos_azimuth = ((declination.sin() - elevation.sin() * lat.sin())
        / (elevation.cos().max(1e-5) * lat.cos().max(1e-5)))
    .clamp(-1.0, 1.0);
    let mut azimuth = cos_azimuth.acos();
    if hour_angle.sin() > 0.0 {
        azimuth = TAU - azimuth;
    }
    let azimuth_degrees = (azimuth.to_degrees() + azimuth_offset_degrees).rem_euclid(360.0);
    let (sunrise_hour, sunset_hour) =
        daylight_window_from_declination(latitude_degrees, declination_degrees);

    SolarSample {
        direction: direction_from_alt_az(elevation_degrees, azimuth_degrees),
        elevation_degrees,
        azimuth_degrees,
        sunrise_hour,
        sunset_hour,
    }
}

fn daylight_window_from_declination(latitude_degrees: f32, declination_degrees: f32) -> (f32, f32) {
    let lat = latitude_degrees.to_radians();
    let declination = declination_degrees.to_radians();
    let raw = -lat.tan() * declination.tan();

    if raw <= -1.0 {
        (0.0, 24.0)
    } else if raw >= 1.0 {
        (12.0, 12.0)
    } else {
        let hour_angle = raw.acos();
        let hours = hour_angle * 12.0 / PI;
        (12.0 - hours, 12.0 + hours)
    }
}

fn direction_from_alt_az(elevation_degrees: f32, azimuth_degrees: f32) -> Vec3 {
    let elevation = elevation_degrees.to_radians();
    let azimuth = azimuth_degrees.to_radians();
    let east = elevation.cos() * azimuth.sin();
    let north = elevation.cos() * azimuth.cos();
    let up = elevation.sin();
    -Vec3::new(east, up, north).normalize_or_zero()
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    if (edge1 - edge0).abs() <= f32::EPSILON {
        return f32::from(value >= edge1);
    }

    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn triangle_peak(value: f32, center: f32, start: f32, end: f32) -> f32 {
    if value <= start || value >= end {
        return 0.0;
    }
    if value <= center {
        (value - start) / (center - start).max(1e-4)
    } else {
        (end - value) / (end - center).max(1e-4)
    }
}

#[cfg(test)]
#[path = "celestial_tests.rs"]
mod tests;
