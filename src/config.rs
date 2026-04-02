use bevy::prelude::*;

use crate::{CelestialSettings, DayPhaseBoundaries, LightingProfile, TimeOverride, TimeWrapMode};

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct ManagedLightConfig {
    pub auto_spawn: bool,
}

impl Default for ManagedLightConfig {
    fn default() -> Self {
        Self { auto_spawn: true }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct ShadowConfig {
    pub sun_min_elevation_degrees: f32,
    pub sun_min_illuminance_lux: f32,
    pub moon_shadows_enabled: bool,
    pub moon_min_elevation_degrees: f32,
    pub moon_min_illuminance_lux: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            sun_min_elevation_degrees: 0.5,
            sun_min_illuminance_lux: 75.0,
            moon_shadows_enabled: true,
            moon_min_elevation_degrees: 2.0,
            moon_min_illuminance_lux: 0.02,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct SmoothingConfig {
    pub continuous_seconds: f32,
    pub jump_seconds: f32,
    pub smooth_scrubs: bool,
}

impl Default for SmoothingConfig {
    fn default() -> Self {
        Self {
            continuous_seconds: 0.18,
            jump_seconds: 0.0,
            smooth_scrubs: false,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct WriteThresholds {
    pub direction_dot_epsilon: f32,
    pub color_epsilon: f32,
    pub illuminance_epsilon: f32,
    pub ambient_brightness_epsilon: f32,
    pub fog_visibility_epsilon: f32,
    pub fog_density_epsilon: f32,
    pub exposure_epsilon: f32,
}

impl Default for WriteThresholds {
    fn default() -> Self {
        Self {
            direction_dot_epsilon: 1e-4,
            color_epsilon: 5e-3,
            illuminance_epsilon: 0.25,
            ambient_brightness_epsilon: 1e-3,
            fog_visibility_epsilon: 0.5,
            fog_density_epsilon: 1e-3,
            exposure_epsilon: 1e-3,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default, Clone)]
pub struct AtmosphereTuning {
    pub scene_units_to_m: f32,
    pub density_multiplier: f32,
    pub environment_map_intensity_scale: f32,
}

impl Default for AtmosphereTuning {
    fn default() -> Self {
        Self {
            scene_units_to_m: 1.0,
            density_multiplier: 1.0,
            environment_map_intensity_scale: 1.0,
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default, Clone)]
pub struct DayNightConfig {
    pub initial_time: f32,
    pub seconds_per_hour: f32,
    pub time_scale: f32,
    pub paused: bool,
    pub wrap_mode: TimeWrapMode,
    pub pending_override: Option<TimeOverride>,
    pub phase_boundaries: DayPhaseBoundaries,
    pub celestial: CelestialSettings,
    pub lighting: LightingProfile,
    pub managed_lights: ManagedLightConfig,
    pub shadows: ShadowConfig,
    pub smoothing: SmoothingConfig,
    pub write_thresholds: WriteThresholds,
    pub atmosphere: AtmosphereTuning,
}

impl Default for DayNightConfig {
    fn default() -> Self {
        Self {
            initial_time: 12.0,
            seconds_per_hour: 120.0,
            time_scale: 1.0,
            paused: false,
            wrap_mode: TimeWrapMode::Loop,
            pending_override: None,
            phase_boundaries: DayPhaseBoundaries::default(),
            celestial: CelestialSettings::default(),
            lighting: LightingProfile::realistic_outdoor(),
            managed_lights: ManagedLightConfig::default(),
            shadows: ShadowConfig::default(),
            smoothing: SmoothingConfig::default(),
            write_thresholds: WriteThresholds::default(),
            atmosphere: AtmosphereTuning::default(),
        }
    }
}

impl DayNightConfig {
    pub fn queue_scrub(&mut self, hour: f32) {
        self.pending_override = Some(TimeOverride::Scrub { hour });
    }

    pub fn queue_advance_hours(&mut self, hours: f32) {
        self.pending_override = Some(TimeOverride::AdvanceHours { hours });
    }

    pub fn fixed_time(mut self, hour: f32) -> Self {
        self.initial_time = hour;
        self.paused = true;
        self.pending_override = None;
        self
    }

    pub fn with_profile(mut self, lighting: LightingProfile) -> Self {
        self.lighting = lighting;
        self
    }
}
