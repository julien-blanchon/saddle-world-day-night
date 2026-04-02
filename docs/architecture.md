# Architecture

## Layering

`saddle-world-day-night` is split into pure logic plus a thin Bevy integration layer.

Pure Rust:

1. `time.rs`
2. `phase.rs`
3. `celestial.rs`
4. `gradient.rs`
5. `lighting.rs`

Bevy-facing:

1. `components.rs`
2. `messages.rs`
3. `config.rs`
4. `systems.rs`
5. `lib.rs`

The pure modules own clock math, phase resolution, keyframe interpolation, sun/moon solving, Kelvin conversion, and lighting resolution. The Bevy layer only initializes resources, advances the runtime, emits messages, and applies resolved state to Bevy components and resources.

## Runtime Flow

```text
DayNightConfig
  -> advance TimeOfDay
  -> resolve DayPhase
  -> solve CelestialState
  -> resolve DayNightLighting
  -> emit phase messages
  -> apply to Sun / Moon / GlobalAmbientLight / DayNightCamera targets
  -> publish DayNightDiagnostics
```

## Schedule Ordering

`DayNightSystems` is intentionally public and chained in this order:

1. `AdvanceTime`
2. `ResolveCelestial`
3. `ResolveLighting`
4. `DetectPhaseTransitions`
5. `ApplyLighting`

This keeps all downstream reads stable inside one frame:

- `CelestialState` always sees the current `TimeOfDay`
- `DayNightLighting` always sees the current `CelestialState`
- phase messages are emitted from the same step that produced the resolved phase
- component writes happen after the pure resources are finalized

The crate accepts injectable activate, deactivate, and update schedules so downstream games can map it into their own state machine or feature pipeline.

## Ownership

### Lights

Two supported patterns exist:

1. Leave `ManagedLightConfig::auto_spawn = true` and let the crate create `Sun` / `Moon` directional lights on demand.
2. Spawn your own directional lights and tag them with `Sun` / `Moon`.

The crate never mutates untagged directional lights.

### Cameras

The crate mutates only entities tagged with `DayNightCamera`.

`DayNightCamera` controls whether the crate:

- writes `DistanceFog`
- writes `VolumetricFog`
- writes `Exposure`
- writes `AtmosphereEnvironmentMapLight`
- inserts missing components automatically
- ensures `Atmosphere` / `AtmosphereSettings`

This keeps camera ownership explicit. If a project wants a split between gameplay and cinematic cameras, it can tag only the outdoor camera that should receive day/night state.

### Ambient Light

`GlobalAmbientLight` is always considered owned by the runtime while active. This is the only global render-facing output the crate writes unconditionally.

## Atmosphere And Fog

The crate does not implement a custom sky renderer. It integrates with Bevy's built-in atmospheric features:

- `Atmosphere`
- `ScatteringMedium`
- `AtmosphereSettings`
- `AtmosphereEnvironmentMapLight`
- `DistanceFog`
- `VolumetricFog`

`ScatteringMedium` assets are cached in an internal resource so repeated camera updates do not churn handles.

Important boundary:

- the crate resolves outdoor lighting and fog hints
- Bevy's atmosphere renderer remains responsible for actual sky and aerial perspective shading

## Smoothing And Write Thresholds

Resolved lighting is smoothed in resource space before touching render-facing components. This avoids visible pops during normal time progression without forcing expensive component rewrites every frame.

Threshold checks are applied before writing:

- light direction
- light color
- illuminance
- ambient brightness
- fog values
- exposure

This keeps the runtime cheap in long-running scenes and makes the diagnostics counters meaningful.

Shadow booleans are not smoothed. They switch to the target state immediately so low-angle or noon shadow state cannot get stuck behind an interpolation factor.

## Phase Messages

The crate emits:

- `DawnStarted`
- `DayStarted`
- `DuskStarted`
- `NightStarted`

Message behavior is intentionally one-directional:

- forward continuous motion and forward jumps emit crossed phase starts in chronological order
- paused frames emit nothing
- backward scrubs or backward jumps do not emit reverse transition messages

That keeps the message surface simple for gameplay consumers. If a game needs reverse-time semantics, it should interpret `TimeOfDay` and `DayPhase` directly instead of relying on inverse messages.

## Day Counter Semantics

`TimeOfDay::elapsed_days` counts completed 24-hour simulation cycles from the current starting point.

Examples:

- start at `0.0`, run to `24.0` worth of elapsed simulation time: `elapsed_days += 1`
- start at `18.0`, run to clock-labeled `00:30`: `elapsed_days` is still `0`
- start at `18.0`, run a full simulated 24 hours: `elapsed_days += 1`

This makes the counter stable for “days elapsed since this session/start point” style gameplay. If a consumer needs a calendar day that increments on each clock midnight, it should track that separately from `TimeOfDay`.

## Testing Strategy

The crate verifies the pure and Bevy boundaries separately:

- pure unit tests for time, phase, celestial math, gradients, lighting, and Kelvin conversion
- Bevy integration tests for plugin build, resource initialization, messages, and managed light behavior
- standalone examples for focused visual use cases
- crate-local lab scenarios for smoke, full cycle, fixed-time scrubbing, phase messages, and write-stability/perf behavior
