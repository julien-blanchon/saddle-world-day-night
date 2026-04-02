use bevy::prelude::*;

use crate::{
    CelestialState, ColorGradient, ColorKeyframe, ScalarGradient, ScalarKeyframe, ShadowConfig,
    SmoothingConfig, TimeOfDay, TimeStepMode, WriteThresholds, gradient::mix_color,
};

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default, Clone)]
pub struct WeatherModulation {
    pub cloud_cover: f32,
    pub haze: f32,
    pub precipitation_dimming: f32,
}

impl Default for WeatherModulation {
    fn default() -> Self {
        Self {
            cloud_cover: 0.0,
            haze: 0.0,
            precipitation_dimming: 0.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct LightingProfile {
    pub sun_illuminance_lux: ScalarGradient,
    pub moon_illuminance_lux: ScalarGradient,
    pub ambient_brightness: ScalarGradient,
    pub exposure_ev100: ScalarGradient,
    pub fog_visibility: ScalarGradient,
    pub fog_density: ScalarGradient,
    pub sun_temperature_kelvin: ScalarGradient,
    pub moon_temperature_kelvin: ScalarGradient,
    pub sun_tint: ColorGradient,
    pub moon_tint: ColorGradient,
    pub ambient_color: ColorGradient,
    pub fog_color: ColorGradient,
}

impl Default for LightingProfile {
    fn default() -> Self {
        Self::realistic_outdoor()
    }
}

impl LightingProfile {
    pub fn realistic_outdoor() -> Self {
        Self {
            sun_illuminance_lux: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 0.0),
                ScalarKeyframe::new(5.5, 0.0),
                ScalarKeyframe::new(6.5, 900.0),
                ScalarKeyframe::new(9.0, 26_000.0),
                ScalarKeyframe::new(12.0, 92_000.0),
                ScalarKeyframe::new(15.0, 42_000.0),
                ScalarKeyframe::new(18.0, 1_100.0),
                ScalarKeyframe::new(19.5, 0.0),
            ]),
            moon_illuminance_lux: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 2.4),
                ScalarKeyframe::new(5.0, 1.8),
                ScalarKeyframe::new(7.0, 0.22),
                ScalarKeyframe::new(18.0, 0.18),
                ScalarKeyframe::new(20.0, 2.4),
            ]),
            ambient_brightness: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 62.0),
                ScalarKeyframe::new(5.5, 74.0),
                ScalarKeyframe::new(6.75, 92.0),
                ScalarKeyframe::new(12.0, 86.0),
                ScalarKeyframe::new(18.0, 78.0),
                ScalarKeyframe::new(20.0, 64.0),
            ]),
            exposure_ev100: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 2.6),
                ScalarKeyframe::new(6.0, 4.8),
                ScalarKeyframe::new(12.0, 13.6),
                ScalarKeyframe::new(18.0, 5.0),
                ScalarKeyframe::new(23.99, 2.6),
            ]),
            fog_visibility: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 240.0),
                ScalarKeyframe::new(6.0, 300.0),
                ScalarKeyframe::new(12.0, 650.0),
                ScalarKeyframe::new(18.0, 340.0),
                ScalarKeyframe::new(23.99, 240.0),
            ]),
            fog_density: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 0.10),
                ScalarKeyframe::new(6.0, 0.08),
                ScalarKeyframe::new(12.0, 0.035),
                ScalarKeyframe::new(18.0, 0.07),
                ScalarKeyframe::new(23.99, 0.10),
            ]),
            sun_temperature_kelvin: ScalarGradient::new(vec![
                ScalarKeyframe::new(0.0, 2_200.0),
                ScalarKeyframe::new(6.0, 2_100.0),
                ScalarKeyframe::new(9.0, 4_600.0),
                ScalarKeyframe::new(12.0, 5_800.0),
                ScalarKeyframe::new(18.0, 2_200.0),
                ScalarKeyframe::new(23.99, 2_200.0),
            ]),
            moon_temperature_kelvin: ScalarGradient::constant(4_200.0),
            sun_tint: ColorGradient::constant(Color::WHITE),
            moon_tint: ColorGradient::constant(Color::srgb(0.78, 0.84, 1.0)),
            ambient_color: ColorGradient::new(vec![
                ColorKeyframe::new(0.0, Color::srgb(0.18, 0.22, 0.34)),
                ColorKeyframe::new(6.0, Color::srgb(0.66, 0.48, 0.34)),
                ColorKeyframe::new(12.0, Color::srgb(0.70, 0.77, 0.92)),
                ColorKeyframe::new(18.0, Color::srgb(0.62, 0.42, 0.32)),
                ColorKeyframe::new(23.99, Color::srgb(0.18, 0.22, 0.34)),
            ]),
            fog_color: ColorGradient::new(vec![
                ColorKeyframe::new(0.0, Color::srgb(0.12, 0.16, 0.28)),
                ColorKeyframe::new(6.0, Color::srgb(0.74, 0.56, 0.42)),
                ColorKeyframe::new(12.0, Color::srgb(0.70, 0.82, 0.96)),
                ColorKeyframe::new(18.0, Color::srgb(0.72, 0.48, 0.34)),
                ColorKeyframe::new(23.99, Color::srgb(0.12, 0.16, 0.28)),
            ]),
        }
    }

    pub fn stylized_saturated() -> Self {
        let mut profile = Self::realistic_outdoor();
        profile.ambient_brightness = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 52.0),
            ScalarKeyframe::new(6.0, 74.0),
            ScalarKeyframe::new(12.0, 118.0),
            ScalarKeyframe::new(18.0, 82.0),
            ScalarKeyframe::new(23.99, 52.0),
        ]);
        profile.sun_tint = ColorGradient::new(vec![
            ColorKeyframe::new(0.0, Color::srgb(0.8, 0.62, 0.72)),
            ColorKeyframe::new(12.0, Color::srgb(1.0, 0.96, 0.82)),
            ColorKeyframe::new(23.99, Color::srgb(0.8, 0.62, 0.72)),
        ]);
        profile.ambient_color = ColorGradient::new(vec![
            ColorKeyframe::new(0.0, Color::srgb(0.18, 0.10, 0.34)),
            ColorKeyframe::new(6.0, Color::srgb(0.90, 0.45, 0.32)),
            ColorKeyframe::new(12.0, Color::srgb(0.38, 0.70, 0.96)),
            ColorKeyframe::new(18.0, Color::srgb(0.98, 0.42, 0.26)),
            ColorKeyframe::new(23.99, Color::srgb(0.18, 0.10, 0.34)),
        ]);
        profile.fog_color = profile.ambient_color.clone();
        profile
    }

    pub fn overcast() -> Self {
        let mut profile = Self::realistic_outdoor();
        profile.sun_illuminance_lux = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 0.0),
            ScalarKeyframe::new(6.0, 400.0),
            ScalarKeyframe::new(12.0, 18_000.0),
            ScalarKeyframe::new(18.0, 400.0),
            ScalarKeyframe::new(23.99, 0.0),
        ]);
        profile.ambient_brightness = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 2.4),
            ScalarKeyframe::new(6.0, 6.0),
            ScalarKeyframe::new(12.0, 30.0),
            ScalarKeyframe::new(18.0, 7.0),
            ScalarKeyframe::new(23.99, 2.4),
        ]);
        profile.fog_density = ScalarGradient::constant(0.24);
        profile.fog_visibility = ScalarGradient::constant(170.0);
        profile.ambient_color = ColorGradient::constant(Color::srgb(0.56, 0.62, 0.68));
        profile.fog_color = ColorGradient::constant(Color::srgb(0.52, 0.58, 0.64));
        profile
    }

    pub fn harsh_desert() -> Self {
        let mut profile = Self::realistic_outdoor();
        profile.sun_illuminance_lux = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 0.0),
            ScalarKeyframe::new(6.0, 1_200.0),
            ScalarKeyframe::new(12.0, 110_000.0),
            ScalarKeyframe::new(18.0, 1_200.0),
            ScalarKeyframe::new(23.99, 0.0),
        ]);
        profile.exposure_ev100 = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 9.0),
            ScalarKeyframe::new(12.0, 15.0),
            ScalarKeyframe::new(23.99, 9.0),
        ]);
        profile.fog_density = ScalarGradient::constant(0.06);
        profile.fog_visibility = ScalarGradient::constant(900.0);
        profile.sun_temperature_kelvin = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 2_200.0),
            ScalarKeyframe::new(6.0, 3_200.0),
            ScalarKeyframe::new(12.0, 6_200.0),
            ScalarKeyframe::new(18.0, 3_000.0),
            ScalarKeyframe::new(23.99, 2_200.0),
        ]);
        profile.ambient_color = ColorGradient::constant(Color::srgb(0.74, 0.62, 0.42));
        profile.fog_color = ColorGradient::constant(Color::srgb(0.88, 0.76, 0.54));
        profile
    }

    pub fn moonlit_night() -> Self {
        let mut profile = Self::realistic_outdoor();
        profile.moon_illuminance_lux = ScalarGradient::constant(2.8);
        profile.ambient_brightness = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 70.0),
            ScalarKeyframe::new(6.0, 78.0),
            ScalarKeyframe::new(12.0, 92.0),
            ScalarKeyframe::new(18.0, 78.0),
            ScalarKeyframe::new(23.99, 70.0),
        ]);
        profile.exposure_ev100 = ScalarGradient::new(vec![
            ScalarKeyframe::new(0.0, 2.0),
            ScalarKeyframe::new(6.0, 3.4),
            ScalarKeyframe::new(12.0, 11.5),
            ScalarKeyframe::new(18.0, 3.4),
            ScalarKeyframe::new(23.99, 2.0),
        ]);
        profile.moon_tint = ColorGradient::constant(Color::srgb(0.70, 0.82, 1.0));
        profile.ambient_color = ColorGradient::new(vec![
            ColorKeyframe::new(0.0, Color::srgb(0.22, 0.28, 0.42)),
            ColorKeyframe::new(6.0, Color::srgb(0.46, 0.36, 0.32)),
            ColorKeyframe::new(12.0, Color::srgb(0.60, 0.68, 0.88)),
            ColorKeyframe::new(18.0, Color::srgb(0.44, 0.34, 0.30)),
            ColorKeyframe::new(23.99, Color::srgb(0.22, 0.28, 0.42)),
        ]);
        profile
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default, Clone)]
pub struct DayNightLighting {
    pub sun_color: Color,
    pub sun_illuminance_lux: f32,
    pub moon_color: Color,
    pub moon_illuminance_lux: f32,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
    pub fog_color: Color,
    pub fog_visibility: f32,
    pub fog_density: f32,
    pub volumetric_ambient_intensity: f32,
    pub suggested_exposure_ev100: f32,
    pub star_visibility: f32,
    pub daylight_factor: f32,
    pub night_factor: f32,
    pub golden_hour_factor: f32,
    pub twilight_factor: f32,
    pub sun_shadows_enabled: bool,
    pub moon_shadows_enabled: bool,
}

impl Default for DayNightLighting {
    fn default() -> Self {
        Self {
            sun_color: Color::WHITE,
            sun_illuminance_lux: 90_000.0,
            moon_color: Color::srgb(0.78, 0.84, 1.0),
            moon_illuminance_lux: 0.1,
            ambient_color: Color::srgb(0.68, 0.74, 0.88),
            ambient_brightness: 42.0,
            fog_color: Color::srgb(0.70, 0.82, 0.96),
            fog_visibility: 650.0,
            fog_density: 0.045,
            volumetric_ambient_intensity: 0.52,
            suggested_exposure_ev100: 14.4,
            star_visibility: 0.0,
            daylight_factor: 1.0,
            night_factor: 0.0,
            golden_hour_factor: 0.0,
            twilight_factor: 0.0,
            sun_shadows_enabled: true,
            moon_shadows_enabled: false,
        }
    }
}

impl DayNightLighting {
    pub fn lerp_to(&self, target: &Self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            sun_color: mix_color(self.sun_color, target.sun_color, factor),
            sun_illuminance_lux: lerp(self.sun_illuminance_lux, target.sun_illuminance_lux, factor),
            moon_color: mix_color(self.moon_color, target.moon_color, factor),
            moon_illuminance_lux: lerp(
                self.moon_illuminance_lux,
                target.moon_illuminance_lux,
                factor,
            ),
            ambient_color: mix_color(self.ambient_color, target.ambient_color, factor),
            ambient_brightness: lerp(self.ambient_brightness, target.ambient_brightness, factor),
            fog_color: mix_color(self.fog_color, target.fog_color, factor),
            fog_visibility: lerp(self.fog_visibility, target.fog_visibility, factor),
            fog_density: lerp(self.fog_density, target.fog_density, factor),
            volumetric_ambient_intensity: lerp(
                self.volumetric_ambient_intensity,
                target.volumetric_ambient_intensity,
                factor,
            ),
            suggested_exposure_ev100: lerp(
                self.suggested_exposure_ev100,
                target.suggested_exposure_ev100,
                factor,
            ),
            star_visibility: lerp(self.star_visibility, target.star_visibility, factor),
            daylight_factor: lerp(self.daylight_factor, target.daylight_factor, factor),
            night_factor: lerp(self.night_factor, target.night_factor, factor),
            golden_hour_factor: lerp(self.golden_hour_factor, target.golden_hour_factor, factor),
            twilight_factor: lerp(self.twilight_factor, target.twilight_factor, factor),
            sun_shadows_enabled: target.sun_shadows_enabled,
            moon_shadows_enabled: target.moon_shadows_enabled,
        }
    }

    pub fn approx_eq(&self, other: &Self, thresholds: &WriteThresholds) -> bool {
        color_distance(self.sun_color, other.sun_color) <= thresholds.color_epsilon
            && (self.sun_illuminance_lux - other.sun_illuminance_lux).abs()
                <= thresholds.illuminance_epsilon
            && color_distance(self.moon_color, other.moon_color) <= thresholds.color_epsilon
            && (self.moon_illuminance_lux - other.moon_illuminance_lux).abs()
                <= thresholds.illuminance_epsilon
            && color_distance(self.ambient_color, other.ambient_color) <= thresholds.color_epsilon
            && (self.ambient_brightness - other.ambient_brightness).abs()
                <= thresholds.ambient_brightness_epsilon
            && color_distance(self.fog_color, other.fog_color) <= thresholds.color_epsilon
            && (self.fog_visibility - other.fog_visibility).abs()
                <= thresholds.fog_visibility_epsilon
            && (self.fog_density - other.fog_density).abs() <= thresholds.fog_density_epsilon
            && (self.suggested_exposure_ev100 - other.suggested_exposure_ev100).abs()
                <= thresholds.exposure_epsilon
            && self.sun_shadows_enabled == other.sun_shadows_enabled
            && self.moon_shadows_enabled == other.moon_shadows_enabled
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default, Clone)]
pub struct DayNightDiagnostics {
    pub current_time: f32,
    pub elapsed_days: u32,
    pub current_phase: crate::DayPhase,
    pub last_step_mode: TimeStepMode,
    pub last_phase_change: Option<crate::DayPhase>,
    pub phase_history: Vec<crate::DayPhase>,
    pub phase_message_count: u64,
    pub sun_writes: u64,
    pub moon_writes: u64,
    pub ambient_writes: u64,
    pub fog_writes: u64,
    pub exposure_writes: u64,
    pub environment_map_writes: u64,
}

impl Default for DayNightDiagnostics {
    fn default() -> Self {
        Self {
            current_time: 12.0,
            elapsed_days: 0,
            current_phase: crate::DayPhase::Day,
            last_step_mode: TimeStepMode::Idle,
            last_phase_change: None,
            phase_history: Vec::new(),
            phase_message_count: 0,
            sun_writes: 0,
            moon_writes: 0,
            ambient_writes: 0,
            fog_writes: 0,
            exposure_writes: 0,
            environment_map_writes: 0,
        }
    }
}

pub fn resolve_lighting(
    time_of_day: TimeOfDay,
    celestial: &CelestialState,
    profile: &LightingProfile,
    weather: &WeatherModulation,
    shadows: &ShadowConfig,
) -> DayNightLighting {
    let hour = time_of_day.cyclic_hour();
    let cloud_cover = weather.cloud_cover.clamp(0.0, 1.0);
    let haze = weather.haze.clamp(0.0, 1.0);
    let precipitation = weather.precipitation_dimming.clamp(0.0, 1.0);
    let sun_visibility = sun_visibility_factor(celestial.sun_elevation_degrees);
    let moon_visibility = moon_visibility_factor(celestial.moon_elevation_degrees);

    let sun_illuminance_lux = (profile.sun_illuminance_lux.sample(hour)
        * sun_visibility
        * (1.0 - 0.72 * cloud_cover)
        * (1.0 - 0.58 * precipitation))
        .max(0.0);
    let moon_illuminance_lux = (profile.moon_illuminance_lux.sample(hour)
        * moon_visibility
        * celestial.night_factor
        * celestial.moon_phase.brightness_factor()
        * (1.0 - 0.55 * cloud_cover))
        .max(0.0);

    let ambient_brightness = (profile.ambient_brightness.sample(hour)
        * (1.0 - 0.30 * precipitation)
        * (1.0 - 0.08 * cloud_cover + 0.18 * haze))
        .max(0.0);
    let fog_visibility =
        (profile.fog_visibility.sample(hour) * (1.0 - 0.52 * haze) * (1.0 - 0.35 * precipitation))
            .max(24.0);
    let fog_density =
        (profile.fog_density.sample(hour) * (1.0 + 1.4 * haze + 1.2 * precipitation)).max(0.0);
    let suggested_exposure_ev100 =
        profile.exposure_ev100.sample(hour) - cloud_cover * 0.55 - precipitation * 0.35
            + haze * 0.10;

    let sun_color = multiply_color(
        kelvin_to_color(profile.sun_temperature_kelvin.sample(hour)),
        profile.sun_tint.sample(hour),
    );
    let moon_color = multiply_color(
        kelvin_to_color(profile.moon_temperature_kelvin.sample(hour)),
        profile.moon_tint.sample(hour),
    );
    let ambient_color = mix_color(
        profile.ambient_color.sample(hour),
        Color::srgb(0.56, 0.58, 0.62),
        cloud_cover * 0.35,
    );
    let fog_color = mix_color(
        profile.fog_color.sample(hour),
        Color::srgb(0.55, 0.57, 0.60),
        (cloud_cover * 0.45 + haze * 0.35).clamp(0.0, 1.0),
    )
    .with_alpha((0.18 + fog_density * 0.35).clamp(0.0, 1.0));

    DayNightLighting {
        sun_color,
        sun_illuminance_lux,
        moon_color,
        moon_illuminance_lux,
        ambient_color,
        ambient_brightness,
        fog_color,
        fog_visibility,
        fog_density,
        volumetric_ambient_intensity: (ambient_brightness / 80.0).clamp(0.0, 2.0),
        suggested_exposure_ev100,
        star_visibility: (celestial.star_visibility * (1.0 - 0.65 * cloud_cover)).clamp(0.0, 1.0),
        daylight_factor: celestial.daylight_factor,
        night_factor: celestial.night_factor,
        golden_hour_factor: celestial.golden_hour_factor,
        twilight_factor: celestial.twilight_factor,
        sun_shadows_enabled: celestial.sun_elevation_degrees >= shadows.sun_min_elevation_degrees
            && sun_illuminance_lux >= shadows.sun_min_illuminance_lux,
        moon_shadows_enabled: shadows.moon_shadows_enabled
            && celestial.moon_elevation_degrees >= shadows.moon_min_elevation_degrees
            && moon_illuminance_lux >= shadows.moon_min_illuminance_lux,
    }
}

pub fn smooth_lighting(
    current: &DayNightLighting,
    target: &DayNightLighting,
    smoothing: &SmoothingConfig,
    step_mode: TimeStepMode,
    delta_seconds: f32,
) -> DayNightLighting {
    let smoothing_seconds = match step_mode {
        TimeStepMode::Idle => smoothing.continuous_seconds,
        TimeStepMode::Continuous => smoothing.continuous_seconds,
        TimeStepMode::Scrub if smoothing.smooth_scrubs => smoothing.jump_seconds,
        TimeStepMode::AdvanceJump => smoothing.jump_seconds,
        _ => 0.0,
    };

    if smoothing_seconds <= 0.0 || delta_seconds <= 0.0 {
        return target.clone();
    }

    let factor = 1.0 - (-delta_seconds / smoothing_seconds.max(1e-4)).exp();
    current.lerp_to(target, factor)
}

pub fn kelvin_to_color(kelvin: f32) -> Color {
    let temperature = (kelvin / 100.0).clamp(10.0, 400.0);
    let red = if temperature <= 66.0 {
        255.0
    } else {
        329.698_73 * (temperature - 60.0).powf(-0.133_204_76)
    };
    let green = if temperature <= 66.0 {
        99.470_8 * temperature.ln() - 161.119_57
    } else {
        288.122_16 * (temperature - 60.0).powf(-0.075_514_846)
    };
    let blue = if temperature >= 66.0 {
        255.0
    } else if temperature <= 19.0 {
        0.0
    } else {
        138.517_73 * (temperature - 10.0).ln() - 305.044_8
    };

    Color::srgb(
        (red / 255.0).clamp(0.0, 1.0),
        (green / 255.0).clamp(0.0, 1.0),
        (blue / 255.0).clamp(0.0, 1.0),
    )
}

fn sun_visibility_factor(elevation_degrees: f32) -> f32 {
    smoothstep(-6.0, 8.0, elevation_degrees).powf(1.15)
}

fn moon_visibility_factor(elevation_degrees: f32) -> f32 {
    smoothstep(-4.0, 7.0, elevation_degrees).powf(1.05)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    if (edge1 - edge0).abs() <= f32::EPSILON {
        return f32::from(value >= edge1);
    }

    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(left: f32, right: f32, factor: f32) -> f32 {
    left + (right - left) * factor
}

fn multiply_color(left: Color, right: Color) -> Color {
    let left = LinearRgba::from(left);
    let right = LinearRgba::from(right);
    Color::LinearRgba(LinearRgba {
        red: left.red * right.red,
        green: left.green * right.green,
        blue: left.blue * right.blue,
        alpha: left.alpha * right.alpha,
    })
}

fn color_distance(left: Color, right: Color) -> f32 {
    let left = LinearRgba::from(left);
    let right = LinearRgba::from(right);
    (left.red - right.red).abs()
        + (left.green - right.green).abs()
        + (left.blue - right.blue).abs()
        + (left.alpha - right.alpha).abs()
}

#[cfg(test)]
#[path = "lighting_tests.rs"]
mod tests;
