mod celestial;
mod components;
mod config;
mod gradient;
mod lighting;
mod messages;
mod phase;
mod systems;
mod time;

pub use celestial::{
    CelestialModel, CelestialSettings, CelestialState, MoonPhase, SeasonSettings,
    solar_daylight_window, solve_celestial_state,
};
pub use components::{DayNightCamera, Moon, Sun};
pub use config::{
    AtmosphereTuning, DayNightConfig, ManagedLightConfig, ShadowConfig, SmoothingConfig,
    WriteThresholds,
};
pub use gradient::{ColorGradient, ColorKeyframe, ScalarGradient, ScalarKeyframe};
pub use lighting::{
    DayNightDiagnostics, DayNightLighting, LightingProfile, WeatherModulation, kelvin_to_color,
    resolve_lighting,
};
pub use messages::{DawnStarted, DayStarted, DuskStarted, NightStarted};
pub use phase::{DayPhase, DayPhaseBoundaries};
pub use time::{
    TimeOfDay, TimeOverride, TimeStep, TimeStepMode, TimeWrapMode, advance_by_hours,
    advance_continuous, apply_time_override,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DayNightSystems {
    AdvanceTime,
    ResolveCelestial,
    ResolveLighting,
    DetectPhaseTransitions,
    ApplyLighting,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct DayNightPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub config: DayNightConfig,
}

impl DayNightPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            config: DayNightConfig::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_config(mut self, config: DayNightConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for DayNightPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.insert_resource(self.config.clone())
            .init_resource::<TimeOfDay>()
            .init_resource::<CelestialState>()
            .init_resource::<DayNightLighting>()
            .init_resource::<DayNightDiagnostics>()
            .init_resource::<WeatherModulation>()
            .init_resource::<systems::AtmosphereAssetCache>()
            .init_resource::<systems::DayNightRuntimeState>()
            .init_resource::<GlobalAmbientLight>()
            .add_message::<DawnStarted>()
            .add_message::<DayStarted>()
            .add_message::<DuskStarted>()
            .add_message::<NightStarted>()
            .register_type::<AtmosphereTuning>()
            .register_type::<CelestialModel>()
            .register_type::<CelestialSettings>()
            .register_type::<CelestialState>()
            .register_type::<ColorGradient>()
            .register_type::<ColorKeyframe>()
            .register_type::<DayNightCamera>()
            .register_type::<DayNightConfig>()
            .register_type::<DayNightDiagnostics>()
            .register_type::<DayNightLighting>()
            .register_type::<DayPhase>()
            .register_type::<DayPhaseBoundaries>()
            .register_type::<LightingProfile>()
            .register_type::<ManagedLightConfig>()
            .register_type::<Moon>()
            .register_type::<MoonPhase>()
            .register_type::<ScalarGradient>()
            .register_type::<ScalarKeyframe>()
            .register_type::<SeasonSettings>()
            .register_type::<ShadowConfig>()
            .register_type::<SmoothingConfig>()
            .register_type::<Sun>()
            .register_type::<TimeOfDay>()
            .register_type::<TimeOverride>()
            .register_type::<TimeStep>()
            .register_type::<TimeStepMode>()
            .register_type::<TimeWrapMode>()
            .register_type::<WeatherModulation>()
            .register_type::<WriteThresholds>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    DayNightSystems::AdvanceTime,
                    DayNightSystems::ResolveCelestial,
                    DayNightSystems::ResolveLighting,
                    DayNightSystems::DetectPhaseTransitions,
                    DayNightSystems::ApplyLighting,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                systems::advance_time
                    .in_set(DayNightSystems::AdvanceTime)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::resolve_celestial_state
                    .in_set(DayNightSystems::ResolveCelestial)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::resolve_lighting_state
                    .in_set(DayNightSystems::ResolveLighting)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::detect_phase_transitions
                    .in_set(DayNightSystems::DetectPhaseTransitions)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::ensure_managed_lights,
                    systems::apply_managed_sun,
                    systems::apply_managed_moon,
                    systems::apply_global_ambient_and_cameras,
                    systems::publish_diagnostics,
                )
                    .chain()
                    .in_set(DayNightSystems::ApplyLighting)
                    .run_if(systems::runtime_is_active),
            );
    }
}
