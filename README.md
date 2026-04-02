# Saddle World Day Night

Reusable time-of-day and outdoor-lighting runtime for Bevy. The crate owns clock progression, named day phases, sun/moon direction solving, ambient and direct-light resolution, optional camera fog and exposure hints, and optional Bevy atmosphere hooks.

It stays project-agnostic: no `game_core`, no screen/state vocabulary, and no gameplay rules. Consumers read the resources and messages this crate publishes to decide what night means for their own game.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-world-day-night = { git = "https://github.com/julien-blanchon/saddle-world-day-night" }
```

```rust
use bevy::prelude::*;
use saddle_world_day_night::{DayNightCamera, DayNightConfig, DayNightPlugin, Moon, Sun};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<DemoState>()
        .add_plugins(DayNightPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            Update,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Outdoor Sun"),
        Sun,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Outdoor Moon"),
        Moon,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Outdoor Camera"),
        Camera3d::default(),
        DayNightCamera::default(),
        Transform::from_xyz(-10.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

For examples and labs, `DayNightPlugin::default()` is the always-on entrypoint. It activates on `PostStartup`, never deactivates, and updates in `Update`.

## Ownership Model

- `Sun` / `Moon`: the crate only mutates directional lights tagged with these marker components. With the default config it will auto-spawn them if they do not exist.
- `DayNightCamera`: the crate only mutates cameras tagged with this marker. Untagged cameras are ignored.
- `GlobalAmbientLight`: always driven by the crate while the runtime is active.
- `DistanceFog`, `VolumetricFog`, `Exposure`, `AtmosphereEnvironmentMapLight`, `Atmosphere`, and `AtmosphereSettings`: only inserted or mutated on tagged `DayNightCamera` entities, and only if the corresponding `DayNightCamera` flags allow it.
- Consumers own gameplay meaning. Read `TimeOfDay`, `CelestialState`, `DayNightLighting`, `DayNightDiagnostics`, and the phase messages to drive schedules, AI, audio, weather, UI, or spawning.

`TimeOfDay::elapsed_days` counts completed 24-hour simulation cycles from the current starting point. If you boot at `initial_time = 18.0`, the first increment happens after a full simulated 24 hours, not at the next clock-labeled midnight.

## Public API

| Type | Purpose |
| --- | --- |
| `DayNightPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `DayNightSystems` | Public ordering hooks: `AdvanceTime`, `ResolveCelestial`, `ResolveLighting`, `DetectPhaseTransitions`, `ApplyLighting` |
| `DayNightConfig` | Top-level runtime configuration |
| `TimeOfDay` | Current hour plus completed-cycle counter |
| `TimeOverride`, `TimeStep`, `TimeStepMode`, `TimeWrapMode` | Clock-control and timing helpers |
| `DayPhase`, `DayPhaseBoundaries` | Named phases and configurable boundaries |
| `CelestialSettings`, `CelestialModel`, `SeasonSettings` | Sun/moon path configuration |
| `CelestialState`, `MoonPhase` | Resolved read-only celestial output |
| `LightingProfile`, `WeatherModulation`, `DayNightLighting`, `DayNightDiagnostics` | Lighting authoring inputs plus resolved output and diagnostics |
| `Sun`, `Moon`, `DayNightCamera` | Opt-in components for managed lights and managed cameras |
| `DawnStarted`, `DayStarted`, `DuskStarted`, `NightStarted` | Phase transition messages |
| `ScalarGradient`, `ColorGradient` and keyframes | Authored time-based curves for intensity, color, fog, and exposure |
| `kelvin_to_color`, `solve_celestial_state`, `resolve_lighting`, `solar_daylight_window` | Pure helpers useful in tools or tests |

## Presets And Modes

- `LightingProfile::realistic_outdoor()`
- `LightingProfile::stylized_saturated()`
- `LightingProfile::overcast()`
- `LightingProfile::harsh_desert()`
- `LightingProfile::moonlit_night()`

Common authoring shortcuts:

- `DayNightConfig::fixed_time(hour)` pauses the clock and scrubs to an exact hour.
- `DayNightConfig::with_profile(profile)` swaps the lighting profile.
- `DayNightConfig::queue_scrub(hour)` and `queue_advance_hours(hours)` request exact jumps on the next update.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal outdoor scene with default managed camera/light ownership | `cargo run -p saddle-world-day-night-example-basic` |
| `full_cycle` | Faster cycle with live overlay for time, phase, elevation, lighting, and diagnostics | `cargo run -p saddle-world-day-night-example-full-cycle` |
| `latitude` | Latitude-aware sun path and seasonal day-length shaping | `cargo run -p saddle-world-day-night-example-latitude` |
| `fixed_time` | Frozen stylized golden-hour art direction | `cargo run -p saddle-world-day-night-example-fixed-time` |
| `atmosphere` | Camera-side atmosphere, exposure, bloom, and environment-map-light integration | `cargo run -p saddle-world-day-night-example-atmosphere` |

## Crate-Local Lab

The workspace includes a crate-local lab app at `shared/world/saddle-world-day-night/examples/lab`:

```bash
cargo run -p saddle-world-day-night-lab
```

E2E verification commands:

```bash
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_smoke
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_full_cycle
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_fixed_time_scrub
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_phase_messages
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_performance
```

## BRP

Useful BRP commands against the lab:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-day-night-lab
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::time::TimeOfDay
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::celestial::CelestialState
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::lighting::DayNightLighting
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::lighting::DayNightDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/day_night_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
