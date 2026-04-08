# Configuration

## `DayNightConfig`

| Field | Type | Default | Valid Range | Effect | Practical Advice |
| --- | --- | --- | --- | --- | --- |
| `initial_time` | `f32` | `12.0` | usually `0.0..=24.0` | Starting clock hour | Use `5.5` or `6.0` for dawn-start scenes; use `18.0+` for night-start scenes |
| `seconds_per_hour` | `f32` | `120.0` | `> 0` | Real seconds required for one simulated hour | Lower is faster. `0.5` is useful for labs; `60-180` is more game-like |
| `time_scale` | `f32` | `1.0` | any finite `f32` | Multiplies clock speed | Use for global fast-forward or slow motion |
| `paused` | `bool` | `false` | boolean | Stops continuous advancement | Pair with `queue_scrub` for exact-time art direction |
| `wrap_mode` | `TimeWrapMode` | `Loop` | enum | Loops through 24 hours or clamps at `24.0` | `Loop` is the normal outdoor-cycle mode; `Clamp` is useful for one-shot transitions |
| `pending_override` | `Option<TimeOverride>` | `None` | optional | One-shot scrub or jump request consumed on the next update | Prefer the helper methods instead of mutating this field directly |
| `phase_boundaries` | `DayPhaseBoundaries` | default dawn/day/dusk/night starts | ordered hours | Defines named day-phase transitions | Tune for readability first, realism second |
| `celestial` | `CelestialSettings` | simple arc + default moon offsets | nested config | Controls sun path, moon offset, and lunar cycle | Use `LatitudeAware` only when the day-length variation matters |
| `lighting` | `LightingProfile` | `realistic_outdoor()` | nested config | Curves for illuminance, color, fog, and exposure | Start from a preset and then tune individual curves |
| `managed_lights` | `ManagedLightConfig` | auto-spawn enabled | nested config | Chooses whether missing `Sun` / `Moon` lights are spawned | Disable auto-spawn if composition already owns the light entities |
| `global_ambient` | `GlobalAmbientConfig` | ambient writes enabled | nested config | Chooses whether the crate writes `GlobalAmbientLight` | Disable it when a game owns ambient lighting separately from day/night |
| `shadows` | `ShadowConfig` | low-angle guards enabled | nested config | Enables and gates sun/moon shadow output | Raise the sun minimum angle if dawn/dusk acne is too visible |
| `smoothing` | `SmoothingConfig` | light continuous smoothing, jump smoothing off | nested config | Resource-space interpolation between resolved lighting states | Leave continuous smoothing on; enable jump smoothing only if scrubs look harsh |
| `write_thresholds` | `WriteThresholds` | small epsilons | nested config | Skips writing nearly unchanged values | Increase thresholds only if diagnostics show excessive churn |
| `atmosphere` | `AtmosphereTuning` | neutral scale factors | nested config | Camera-side atmosphere and environment-map tuning | Treat this as a bridge into Bevy's built-in atmosphere, not a sky model |

Helpful builder-style helpers:

- `DayNightConfig::fixed_time(hour)`
- `DayNightConfig::with_profile(profile)`
- `DayNightConfig::queue_scrub(hour)`
- `DayNightConfig::queue_advance_hours(hours)`

## `TimeWrapMode`

| Variant | Effect |
| --- | --- |
| `Loop` | Clock wraps back into `0.0..24.0` and `elapsed_days` counts full simulated 24-hour cycles |
| `Clamp` | Clock stops at `24.0` and `elapsed_days` does not advance automatically |

## `DayPhaseBoundaries`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `dawn_starts` | `f32` | `5.5` | `>= 0` | Beginning of `Dawn` |
| `day_starts` | `f32` | `7.0` | `> dawn_starts` | Beginning of `Day` |
| `dusk_starts` | `f32` | `18.0` | `> day_starts` | Beginning of `Dusk` |
| `night_starts` | `f32` | `19.5` | `> dusk_starts`, `< 24.0` | Beginning of `Night` |

Keep the hours strictly ordered. Night automatically spans from `night_starts` through midnight to `dawn_starts`.

## `CelestialSettings`

| Field | Type | Default | Valid Range | Effect | Advice |
| --- | --- | --- | --- | --- | --- |
| `model` | `CelestialModel` | `SimpleArc { peak_elevation_degrees: 72.0 }` | enum | Chooses the sun-path solver | `SimpleArc` is ideal for stylized or generic games |
| `azimuth_offset_degrees` | `f32` | `0.0` | any finite angle | Rotates the whole path around the horizon | Use when the art direction wants sunrise from a specific world direction |
| `moon_hour_offset` | `f32` | `12.0` | any finite `f32` | Offsets moon path from the sun path in hours | Keep `12.0` for “roughly opposite” or move it for stylized skies |
| `moon_elevation_offset_degrees` | `f32` | `8.0` | any finite angle | Extra moon altitude offset | Helpful when the moon should stay more visible at night |
| `lunar_period_days` | `f32` | `29.53` | `> 0` | Synodic-style moon phase period | Shorten for games that want visible phase change quickly |
| `lunar_phase_offset` | `f32` | `0.0` | any finite `f32` | Starting moon phase offset | Useful when a save file or scenario wants a known moon phase |

### `CelestialModel`

`SimpleArc`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `peak_elevation_degrees` | `f32` | `72.0` | Noon height for the sun path |

`LatitudeAware`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `latitude_degrees` | `f32` | required | Changes noon height and day length |
| `season` | `SeasonSettings` | `SeasonSettings::default()` | Changes declination across the year |

### `SeasonSettings`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `axial_tilt_degrees` | `f32` | `23.4` | Maximum declination swing |
| `season_progress` | `f32` | `0.0` | Fractional progress through the annual cycle |

## `LightingProfile`

`LightingProfile` is a bundle of authored gradients:

| Field | Type | Effect |
| --- | --- | --- |
| `sun_illuminance_lux` | `ScalarGradient` | Direct sun lux curve |
| `moon_illuminance_lux` | `ScalarGradient` | Direct moon lux curve |
| `ambient_brightness` | `ScalarGradient` | `GlobalAmbientLight::brightness` target |
| `exposure_ev100` | `ScalarGradient` | Suggested camera exposure |
| `fog_visibility` | `ScalarGradient` | Visibility distance used for `DistanceFog` |
| `fog_density` | `ScalarGradient` | Volumetric/density hint |
| `sun_temperature_kelvin` | `ScalarGradient` | Sun color temperature |
| `moon_temperature_kelvin` | `ScalarGradient` | Moon color temperature |
| `sun_tint` | `ColorGradient` | Additional authored sun tint |
| `moon_tint` | `ColorGradient` | Additional authored moon tint |
| `ambient_color` | `ColorGradient` | Global ambient color |
| `fog_color` | `ColorGradient` | Fog color |

Preset constructors:

- `LightingProfile::realistic_outdoor()`
- `LightingProfile::stylized_saturated()`
- `LightingProfile::overcast()`
- `LightingProfile::harsh_desert()`
- `LightingProfile::moonlit_night()`

## `ManagedLightConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `auto_spawn` | `bool` | `true` | Spawn missing `Sun` / `Moon` lights automatically |

## `GlobalAmbientConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `apply` | `bool` | `true` | Write the resolved ambient color and brightness into `GlobalAmbientLight` |

## `ShadowConfig`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `sun_min_elevation_degrees` | `f32` | `0.5` | any finite `f32` | Below this sun altitude, sun shadows turn off |
| `sun_min_illuminance_lux` | `f32` | `75.0` | `>= 0` | Below this direct light intensity, sun shadows turn off |
| `moon_shadows_enabled` | `bool` | `true` | boolean | Master moon-shadow toggle |
| `moon_min_elevation_degrees` | `f32` | `2.0` | any finite `f32` | Below this moon altitude, moon shadows turn off |
| `moon_min_illuminance_lux` | `f32` | `0.02` | `>= 0` | Below this moon intensity, moon shadows turn off |

## `SmoothingConfig`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `continuous_seconds` | `f32` | `0.18` | `>= 0` | Smoothing time constant during normal progression |
| `jump_seconds` | `f32` | `0.0` | `>= 0` | Smoothing time constant for advance-jump operations |
| `smooth_scrubs` | `bool` | `false` | boolean | If true, scrubs use `jump_seconds` instead of snapping immediately |

## `WriteThresholds`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `direction_dot_epsilon` | `f32` | `1e-4` | Skip tiny direction updates |
| `color_epsilon` | `f32` | `5e-3` | Skip tiny color updates |
| `illuminance_epsilon` | `f32` | `0.25` | Skip tiny lux updates |
| `ambient_brightness_epsilon` | `f32` | `1e-3` | Skip tiny ambient brightness updates |
| `fog_visibility_epsilon` | `f32` | `0.5` | Skip tiny visibility changes |
| `fog_density_epsilon` | `f32` | `1e-3` | Skip tiny density changes |
| `exposure_epsilon` | `f32` | `1e-3` | Skip tiny exposure changes |

These thresholds only gate render-facing writes. The pure resolved resources are still updated every frame.

## `AtmosphereTuning`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `scene_units_to_m` | `f32` | `1.0` | Written to `AtmosphereSettings::scene_units_to_m` |
| `density_multiplier` | `f32` | `1.0` | Applied when creating the cached earthlike `ScatteringMedium` |
| `environment_map_intensity_scale` | `f32` | `1.0` | Scales the derived `AtmosphereEnvironmentMapLight` intensity |

## `WeatherModulation`

`WeatherModulation` is a runtime resource, not a nested field on `DayNightConfig`, but most consumers will tune it alongside day/night configuration:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `cloud_cover` | `f32` | `0.0` | `0..=1` | Dims direct light and reduces star visibility |
| `haze` | `f32` | `0.0` | `0..=1` | Reduces visibility and thickens fog |
| `precipitation_dimming` | `f32` | `0.0` | `0..=1` | Further dims light and thickens fog |

## `DayNightCamera`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | `bool` | `true` | Master per-camera runtime toggle |
| `apply_distance_fog` | `bool` | `true` | Mutate or insert `DistanceFog` |
| `apply_volumetric_fog` | `bool` | `true` | Mutate or insert `VolumetricFog` |
| `apply_exposure` | `bool` | `true` | Mutate or insert `Exposure` |
| `apply_environment_map_light` | `bool` | `true` | Mutate or insert `AtmosphereEnvironmentMapLight` |
| `insert_missing_components` | `bool` | `true` | Insert render components when absent instead of only mutating existing ones |
| `ensure_atmosphere` | `bool` | `false` | Ensure `Atmosphere` and `AtmosphereSettings` exist for this camera |

## `TimeReactive`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `active_start_hour` | `f32` | `19.0` | `0.0..=24.0` | Hour at which the entity becomes active |
| `active_end_hour` | `f32` | `6.0` | `0.0..=24.0` | Hour at which the entity becomes inactive |

If `active_start_hour > active_end_hour`, the window wraps around midnight (e.g. 19:00–06:00).

Preset constructors:

- `TimeReactive::night_active()` — active 19:00–06:00 (default)
- `TimeReactive::day_active()` — active 06:00–19:00
- `TimeReactive::custom(start, end)` — arbitrary window

The `UpdateTimeReactive` system set runs after `ApplyLighting` and inserts/removes `TimeActive` marker components.

## Tuning Notes

### Physically-Inspired Defaults vs Stylized Authoring

The default profile starts from physically plausible lux and Kelvin ranges, then layers authored gradients on top. For stylized projects:

- start from `stylized_saturated()`
- push `sun_tint`, `ambient_color`, and `fog_color`
- keep lux curves readable even if they are not realistic
- drive post-processing from `suggested_exposure_ev100`, not only from direct light color

### Atmosphere Integration

If you want Bevy's atmospheric ambient and reflections to respond to the day/night state:

1. tag the camera with `DayNightCamera`
2. enable `ensure_atmosphere`
3. keep `apply_environment_map_light` enabled
4. optionally add your own tonemapping, bloom, or color grading

The crate only supplies the outdoor lighting state. It does not replace Bevy's sky rendering or precomputed scattering.

### Fog Ownership

Fog is treated as a camera concern.

- tag only the cameras that should receive outdoor fog
- disable `insert_missing_components` if another system owns the camera-side fog components already
- disable `apply_distance_fog` or `apply_volumetric_fog` selectively if a project wants only one of them managed here

### Exact-Time Art Direction

Use `fixed_time(hour)` for static showcase scenes, title screens, or track intros. The runtime still resolves `CelestialState`, `DayNightLighting`, and camera/light outputs through the same path, so fixed-time scenes remain consistent with fully dynamic scenes.

### Day Counter Semantics

`elapsed_days` is “completed 24-hour runtime cycles,” not “number of clock-midnight boundaries crossed.” If a game needs the latter, derive it separately from `TimeOfDay` and the chosen initial clock.
