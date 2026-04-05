use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct Sun;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct Moon;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct DayNightCamera {
    pub enabled: bool,
    pub apply_distance_fog: bool,
    pub apply_volumetric_fog: bool,
    pub apply_exposure: bool,
    pub apply_environment_map_light: bool,
    pub insert_missing_components: bool,
    pub ensure_atmosphere: bool,
}

impl Default for DayNightCamera {
    fn default() -> Self {
        Self {
            enabled: true,
            apply_distance_fog: true,
            apply_volumetric_fog: true,
            apply_exposure: true,
            apply_environment_map_light: true,
            insert_missing_components: true,
            ensure_atmosphere: false,
        }
    }
}

/// Component for entities that react to the time of day.
///
/// When the current hour is within `[active_start_hour, active_end_hour]` (wrapping around
/// midnight if `active_start_hour > active_end_hour`), the system inserts a [`TimeActive`]
/// marker on the entity. When the time moves outside that window, `TimeActive` is removed.
///
/// Use this for street lamps, window emissions, NPC schedules, or any entity whose behavior
/// should change based on time of day.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct TimeReactive {
    /// Hour at which this entity becomes active (0.0–24.0).
    pub active_start_hour: f32,
    /// Hour at which this entity becomes inactive (0.0–24.0).
    /// If `active_end_hour < active_start_hour`, the window wraps around midnight.
    pub active_end_hour: f32,
}

impl Default for TimeReactive {
    fn default() -> Self {
        Self {
            active_start_hour: 19.0,
            active_end_hour: 6.0,
        }
    }
}

impl TimeReactive {
    /// Creates a time-reactive config for night-active entities (e.g. street lamps).
    /// Active from dusk (19:00) to dawn (6:00).
    pub fn night_active() -> Self {
        Self::default()
    }

    /// Creates a time-reactive config for day-active entities.
    /// Active from dawn (6:00) to dusk (19:00).
    pub fn day_active() -> Self {
        Self {
            active_start_hour: 6.0,
            active_end_hour: 19.0,
        }
    }

    /// Creates a custom time-reactive window.
    pub fn custom(start: f32, end: f32) -> Self {
        Self {
            active_start_hour: start,
            active_end_hour: end,
        }
    }

    /// Returns `true` if the given hour falls within the active window.
    pub fn is_active_at(&self, hour: f32) -> bool {
        let hour = hour.rem_euclid(24.0);
        if self.active_start_hour <= self.active_end_hour {
            // Non-wrapping: e.g. 6:00 to 19:00
            hour >= self.active_start_hour && hour < self.active_end_hour
        } else {
            // Wrapping around midnight: e.g. 19:00 to 6:00
            hour >= self.active_start_hour || hour < self.active_end_hour
        }
    }
}

/// Marker component inserted/removed by the day-night system on entities with [`TimeReactive`].
///
/// Present when the current time is within the entity's active window.
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct TimeActive;
