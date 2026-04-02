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
