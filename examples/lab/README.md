# Day Night Lab

Crate-local standalone lab app for validating the shared `saddle-world-day-night` crate in a real Bevy application.

## Purpose

- verify that the shared crate drives managed sun and moon lights, ambient light, camera fog, exposure, and optional atmosphere hooks in one scene
- keep a deterministic outdoor showcase available for dawn, noon, dusk, and night screenshot gates
- expose time, phase, lighting, message counts, and write counters through an on-screen overlay for BRP and E2E inspection

## Status

Working

## Run

```bash
cargo run -p saddle-world-day-night-lab
```

## E2E

```bash
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_smoke
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_full_cycle
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_fixed_time_scrub
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_phase_messages
cargo run -p saddle-world-day-night-lab --features e2e -- day_night_performance
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-day-night-lab
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::time::TimeOfDay
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::celestial::CelestialState
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_day_night::lighting::DayNightLighting
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/day_night_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Notes

- The lab uses explicit `Sun`, `Moon`, and `DayNightCamera` entities instead of relying on auto-spawn so BRP and E2E can target stable names.
- The lab pins `lunar_phase_offset = 0.5` so the cycle exercises a readable full-moon night instead of a near-black new-moon night.
- The scene palette is intentionally brighter than the crate's minimal examples so dawn, dusk, and moonlit states remain readable in screenshot-backed verification.
