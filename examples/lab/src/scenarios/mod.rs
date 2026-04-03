mod support;

use saddle_bevy_e2e::{action::Action, scenario::Scenario};
use saddle_world_day_night::DayPhase;

use crate::{PerformanceSnapshot, PhaseLog};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "day_night_smoke",
        "day_night_full_cycle",
        "day_night_fixed_time_scrub",
        "day_night_phase_messages",
        "day_night_performance",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "day_night_smoke" => Some(day_night_smoke()),
        "day_night_full_cycle" => Some(day_night_full_cycle()),
        "day_night_fixed_time_scrub" => Some(day_night_fixed_time_scrub()),
        "day_night_phase_messages" => Some(day_night_phase_messages()),
        "day_night_performance" => Some(day_night_performance()),
        _ => None,
    }
}

fn day_night_smoke() -> Scenario {
    Scenario::builder("day_night_smoke")
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
        .then(Action::Screenshot("day_night_smoke".into()))
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

fn day_night_performance() -> Scenario {
    Scenario::builder("day_night_performance")
        .description("Pause the lab at noon and assert that write counters stay stable instead of rewriting identical light and fog values every frame.")
        .then(Action::Custom(Box::new(|world| {
            let mut config = world.resource_mut::<saddle_world_day_night::DayNightConfig>();
            config.paused = true;
            config.queue_scrub(12.0);
        })))
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
