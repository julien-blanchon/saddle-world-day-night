mod support;

use bevy::{
    camera::Exposure,
    light::{AtmosphereEnvironmentMapLight, VolumetricFog},
    pbr::{Atmosphere, AtmosphereSettings, DistanceFog},
    prelude::*,
};
use saddle_bevy_e2e::{action::Action, scenario::Scenario};
use saddle_world_day_night::DayPhase;

use crate::{PerformanceSnapshot, PhaseLog};

#[derive(Resource, Clone, Copy, Default)]
struct WeatherSnapshot {
    clear_sun_lux: f32,
    clear_fog_visibility: f32,
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "day_night_smoke",
        "day_night_full_cycle",
        "day_night_fixed_time_scrub",
        "day_night_phase_messages",
        "day_night_performance",
        "day_night_time_reactive",
        "day_night_weather_modulation",
        "day_night_latitude_model",
        "day_night_camera_hooks",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "day_night_smoke" => Some(day_night_smoke()),
        "day_night_full_cycle" => Some(day_night_full_cycle()),
        "day_night_fixed_time_scrub" => Some(day_night_fixed_time_scrub()),
        "day_night_phase_messages" => Some(day_night_phase_messages()),
        "day_night_performance" => Some(day_night_performance()),
        "day_night_time_reactive" => Some(day_night_time_reactive()),
        "day_night_weather_modulation" => Some(day_night_weather_modulation()),
        "day_night_latitude_model" => Some(day_night_latitude_model()),
        "day_night_camera_hooks" => Some(day_night_camera_hooks()),
        _ => None,
    }
}

fn smoke_launch() -> Scenario {
    day_night_smoke_named("smoke_launch")
}

fn day_night_smoke() -> Scenario {
    day_night_smoke_named("day_night_smoke")
}

fn day_night_smoke_named(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description("Boot the lab, verify resources and managed entities exist, then capture a readable dawn checkpoint.")
        .then(Action::WaitUntil {
            label: "dawn checkpoint".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
                diagnostics.current_phase == DayPhase::Dawn && diagnostics.current_time >= 5.5
            }),
            max_frames: 90,
        })
        .then(Action::Custom(Box::new(|world| {
            assert!(world.contains_resource::<saddle_world_day_night::TimeOfDay>());
            assert!(world.contains_resource::<saddle_world_day_night::CelestialState>());
            assert!(world.contains_resource::<saddle_world_day_night::DayNightLighting>());
            assert!(support::entity_by_name::<saddle_world_day_night::Sun>(world, "Lab Sun").is_some());
            assert!(support::entity_by_name::<saddle_world_day_night::Moon>(world, "Lab Moon").is_some());
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert_eq!(diagnostics.current_phase, DayPhase::Dawn);
            let overlay = support::overlay_text(world).expect("overlay text should exist");
            assert!(overlay.contains("Day Night Lab"));
        })))
        .then(Action::Screenshot(name.into()))
        .build()
}

fn day_night_full_cycle() -> Scenario {
    Scenario::builder("day_night_full_cycle")
        .description("Let the fast lab cycle progress through dawn, noon, dusk, and midnight with hard assertions and screenshots at each step.")
        .then(Action::WaitUntil {
            label: "dawn".into(),
            condition: Box::new(|world| support::time_of_day(world).is_some_and(|time| time.hour >= 6.0)),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world| {
            let lighting = support::lighting(world);
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert_eq!(diagnostics.current_phase, DayPhase::Dawn);
            assert!(lighting.twilight_factor > 0.05);
            assert!(lighting.sun_illuminance_lux > 10.0);
        })))
        .then(Action::Screenshot("dawn".into()))
        .then(Action::WaitUntil {
            label: "noon".into(),
            condition: Box::new(|world| support::time_of_day(world).is_some_and(|time| time.hour >= 12.0)),
            max_frames: 360,
        })
        .then(Action::Custom(Box::new(|world| {
            let lighting = support::lighting(world);
            assert!(lighting.sun_illuminance_lux > 20_000.0);
            assert!(lighting.sun_shadows_enabled);
        })))
        .then(Action::Screenshot("noon".into()))
        .then(Action::WaitUntil {
            label: "dusk".into(),
            condition: Box::new(|world| support::time_of_day(world).is_some_and(|time| time.hour >= 18.0)),
            max_frames: 540,
        })
        .then(Action::Custom(Box::new(|world| {
            let lighting = support::lighting(world);
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert_eq!(diagnostics.current_phase, DayPhase::Dusk);
            assert!(lighting.twilight_factor > 0.05 || lighting.night_factor > 0.1);
        })))
        .then(Action::Screenshot("dusk".into()))
        .then(Action::WaitFrames(220))
        .then(Action::Custom(Box::new(|world| {
            let time = support::time_of_day(world).expect("time resource should exist");
            let lighting = support::lighting(world);
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert_eq!(diagnostics.current_phase, DayPhase::Night);
            assert!(time.hour < 2.0);
            assert!(lighting.star_visibility > 0.25);
            assert!(!lighting.sun_shadows_enabled);
        })))
        .then(Action::Screenshot("midnight".into()))
        .build()
}

fn day_night_fixed_time_scrub() -> Scenario {
    let scrubs = [
        ("scrub_06", 6.0_f32, DayPhase::Dawn),
        ("scrub_12", 12.0_f32, DayPhase::Day),
        ("scrub_18", 18.0_f32, DayPhase::Dusk),
        ("scrub_00", 0.0_f32, DayPhase::Night),
    ];

    let mut builder = Scenario::builder("day_night_fixed_time_scrub")
        .description("Pause the lab and scrub to exact authored times, asserting the resolved phase and lighting each time.");

    for (label, hour, expected_phase) in scrubs {
        builder = builder
            .then(Action::Custom(Box::new(move |world| {
                let mut config = world.resource_mut::<saddle_world_day_night::DayNightConfig>();
                config.paused = true;
                config.queue_scrub(hour);
            })))
            .then(Action::WaitFrames(3))
            .then(Action::Custom(Box::new(move |world| {
                let time = support::time_of_day(world).expect("time resource should exist");
                let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
                assert!((time.hour - hour).abs() < 0.05);
                assert_eq!(diagnostics.current_phase, expected_phase);
            })))
            .then(Action::Screenshot(label.into()))
            .then(Action::WaitFrames(1));
    }

    builder.build()
}

fn day_night_phase_messages() -> Scenario {
    Scenario::builder("day_night_phase_messages")
        .description("Run one fast cycle and verify the phase messages arrive in chronological order exactly once.")
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<PhaseLog>() = PhaseLog::default();
            let mut config = world.resource_mut::<saddle_world_day_night::DayNightConfig>();
            config.paused = false;
            config.seconds_per_hour = 0.5;
        })))
        .then(Action::WaitUntil {
            label: "phase log filled".into(),
            condition: Box::new(|world| world.resource::<PhaseLog>().entries.len() >= 4),
            max_frames: 900,
        })
        .then(Action::Custom(Box::new(|world| {
            let log = world.resource::<PhaseLog>();
            assert_eq!(
                &log.entries[0..4],
                &[DayPhase::Dawn, DayPhase::Day, DayPhase::Dusk, DayPhase::Night]
            );
            assert_eq!(log.dawn_count, 1);
            assert_eq!(log.day_count, 1);
            assert_eq!(log.dusk_count, 1);
            assert_eq!(log.night_count, 1);
        })))
        .then(Action::Screenshot("day_night_phase_messages".into()))
        .build()
}

fn day_night_time_reactive() -> Scenario {
    Scenario::builder("day_night_time_reactive")
        .description("Spawn a TimeReactive entity that is active at night (19:00–06:00), scrub to daytime and assert TimeActive is absent, then scrub to night and assert TimeActive is present.")
        .then(support::pause_and_scrub(14.0))
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world| {
            // Spawn the reactive entity (active window: 19:00–06:00, i.e. night).
            world.spawn((
                Name::new("NightLamp"),
                saddle_world_day_night::TimeReactive {
                    active_start_hour: 19.0,
                    active_end_hour: 6.0,
                },
            ));
        })))
        .then(Action::WaitFrames(2))
        // At 14:00 the lamp should NOT be active.
        .then(Action::Custom(Box::new(|world| {
            let mut q = world.query::<(&Name, Has<saddle_world_day_night::TimeActive>)>();
            let (_, is_active) = q
                .iter(world)
                .find(|(name, _)| name.as_str() == "NightLamp")
                .expect("NightLamp should exist");
            assert!(
                !is_active,
                "TimeActive should be absent during daytime for NightLamp"
            );
        })))
        .then(Action::Screenshot("time_reactive_day".into()))
        // Now scrub to 22:00 – inside the active window.
        .then(support::pause_and_scrub(22.0))
        .then(Action::WaitFrames(2))
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world| {
            let mut q = world.query::<(&Name, Has<saddle_world_day_night::TimeActive>)>();
            let (_, is_active) = q
                .iter(world)
                .find(|(name, _)| name.as_str() == "NightLamp")
                .expect("NightLamp should exist");
            assert!(
                is_active,
                "TimeActive should be present during the active night window"
            );
        })))
        .then(Action::Screenshot("time_reactive_night".into()))
        .build()
}

fn day_night_weather_modulation() -> Scenario {
    Scenario::builder("day_night_weather_modulation")
        .description("Inject WeatherModulation (heavy cloud cover + precipitation dimming) at noon and assert that resolved sun illuminance drops compared to the clear baseline.")
        .then(support::pause_and_scrub(12.0))
        .then(Action::WaitFrames(5))
        // Capture clear-sky baseline.
        .then(Action::Custom(Box::new(|world| {
            let lighting = world.resource::<saddle_world_day_night::DayNightLighting>();
            let clear_sun_lux = lighting.sun_illuminance_lux;
            let clear_fog_visibility = lighting.fog_visibility;
            let _ = lighting;
            world.insert_resource(WeatherSnapshot {
                clear_sun_lux,
                clear_fog_visibility,
            });
        })))
        .then(Action::Screenshot("weather_modulation_clear".into()))
        // Apply heavy overcast modulation.
        .then(Action::Custom(Box::new(|world| {
            let mut modulation = world.resource_mut::<saddle_world_day_night::WeatherModulation>();
            modulation.cloud_cover = 1.0;
            modulation.haze = 0.8;
            modulation.precipitation_dimming = 0.5;
        })))
        .then(Action::WaitFrames(5))
        .then(Action::Custom(Box::new(|world| {
            let lighting = world.resource::<saddle_world_day_night::DayNightLighting>();
            let baseline = *world.resource::<WeatherSnapshot>();
            assert!(
                lighting.sun_illuminance_lux < baseline.clear_sun_lux,
                "cloud cover should dim sun illuminance relative to the clear baseline"
            );
            assert!(
                lighting.fog_visibility < baseline.clear_fog_visibility,
                "heavy haze should reduce fog visibility relative to the clear baseline"
            );
            let modulation = world.resource::<saddle_world_day_night::WeatherModulation>();
            assert!((modulation.cloud_cover - 1.0).abs() < 0.01);
        })))
        .then(Action::Screenshot("weather_modulation_overcast".into()))
        // Reset modulation.
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<saddle_world_day_night::WeatherModulation>() =
                saddle_world_day_night::WeatherModulation::default();
        })))
        .build()
}

fn day_night_performance() -> Scenario {
    Scenario::builder("day_night_performance")
        .description("Pause the lab at noon and assert that write counters stay stable instead of rewriting identical light and fog values every frame.")
        .then(support::pause_and_scrub(12.0))
        .then(Action::WaitFrames(5))
        .then(Action::Custom(Box::new(|world| {
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            *world.resource_mut::<PerformanceSnapshot>() = PerformanceSnapshot {
                sun_writes: diagnostics.sun_writes,
                moon_writes: diagnostics.moon_writes,
                ambient_writes: diagnostics.ambient_writes,
                fog_writes: diagnostics.fog_writes,
                exposure_writes: diagnostics.exposure_writes,
            };
        })))
        .then(Action::WaitFrames(120))
        .then(Action::Custom(Box::new(|world| {
            let before = *world.resource::<PerformanceSnapshot>();
            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert!(diagnostics.sun_writes <= before.sun_writes + 1);
            assert!(diagnostics.moon_writes <= before.moon_writes + 1);
            assert!(diagnostics.ambient_writes <= before.ambient_writes + 1);
            assert!(diagnostics.fog_writes <= before.fog_writes + 2);
            assert!(diagnostics.exposure_writes <= before.exposure_writes + 1);
        })))
        .then(Action::Screenshot("day_night_performance".into()))
        .build()
}

fn day_night_latitude_model() -> Scenario {
    Scenario::builder("day_night_latitude_model")
        .description("Switch the lab from the default simple arc to a latitude-aware summer model, then verify the longer daylight window and capture noon plus evening checkpoints.")
        .then(support::pause_and_scrub(12.0))
        .then(Action::WaitFrames(4))
        .then(Action::Custom(Box::new(|world| {
            let mut config = world.resource_mut::<saddle_world_day_night::DayNightConfig>();
            config.celestial.model = saddle_world_day_night::CelestialModel::LatitudeAware {
                latitude_degrees: 62.0,
                season: saddle_world_day_night::SeasonSettings {
                    season_progress: 0.25,
                    ..default()
                },
            };
            config.queue_scrub(12.0);
        })))
        .then(Action::WaitFrames(4))
        .then(Action::Custom(Box::new(|world| {
            let celestial = world.resource::<saddle_world_day_night::CelestialState>();
            assert!(
                celestial.sunrise_hour < 5.0,
                "summer sunrise should be earlier than the generic 6:00 window"
            );
            assert!(
                celestial.sunset_hour > 19.0,
                "summer sunset should be later than the generic 18:00 window"
            );
            assert!(
                (40.0..60.0).contains(&celestial.sun_elevation_degrees),
                "latitude-aware summer noon should stay in a believable mid-latitude range"
            );
        })))
        .then(Action::Screenshot("latitude_model_noon".into()))
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<saddle_world_day_night::DayNightConfig>()
                .queue_scrub(20.0);
        })))
        .then(Action::WaitFrames(4))
        .then(Action::Custom(Box::new(|world| {
            let celestial = world.resource::<saddle_world_day_night::CelestialState>();
            assert!(
                celestial.sun_elevation_degrees > 2.0,
                "summer evening sun should still be above the horizon around 20:00"
            );
        })))
        .then(Action::Screenshot("latitude_model_evening".into()))
        .build()
}

fn day_night_camera_hooks() -> Scenario {
    Scenario::builder("day_night_camera_hooks")
        .description("Verify the managed lab camera receives fog, exposure, atmosphere, and environment-map hooks, then capture a stable noon checkpoint.")
        .then(support::pause_and_scrub(12.0))
        .then(Action::WaitFrames(5))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::entity_by_name::<Camera>(world, "Lab Camera")
                .expect("Lab Camera should exist");
            let entity = world.entity(camera);
            assert!(entity.contains::<DistanceFog>());
            assert!(entity.contains::<VolumetricFog>());
            assert!(entity.contains::<Exposure>());
            assert!(entity.contains::<Atmosphere>());
            assert!(entity.contains::<AtmosphereSettings>());
            assert!(entity.contains::<AtmosphereEnvironmentMapLight>());

            let diagnostics = world.resource::<saddle_world_day_night::DayNightDiagnostics>();
            assert!(diagnostics.fog_writes > 0);
            assert!(diagnostics.exposure_writes > 0);
            assert!(diagnostics.environment_map_writes > 0);
        })))
        .then(Action::Screenshot("camera_hooks_noon".into()))
        .build()
}
