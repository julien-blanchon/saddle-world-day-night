use bevy::{
    camera::Exposure,
    light::{AtmosphereEnvironmentMapLight, VolumetricFog},
    pbr::{Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff, ScatteringMedium},
    prelude::*,
};

use crate::{
    CelestialState, DawnStarted, DayNightCamera, DayNightConfig, DayNightDiagnostics,
    DayNightLighting, DayStarted, DuskStarted, Moon, NightStarted, Sun, TimeOfDay, TimeStep,
    TimeStepMode, resolve_lighting, solve_celestial_state,
};

#[derive(Resource, Default)]
pub(crate) struct AtmosphereAssetCache {
    earthlike: Option<Handle<ScatteringMedium>>,
    density_multiplier: Option<f32>,
}

#[derive(Resource, Default)]
pub(crate) struct DayNightRuntimeState {
    pub active: bool,
    pub initialized: bool,
    pub lighting_initialized: bool,
    pub last_step: Option<TimeStep>,
    pub last_announced_step: Option<(f64, f64, TimeStepMode)>,
    pub spawned_sun: Option<Entity>,
    pub spawned_moon: Option<Entity>,
}

pub(crate) fn activate_runtime(
    config: Res<DayNightConfig>,
    weather: Res<crate::WeatherModulation>,
    mut runtime: ResMut<DayNightRuntimeState>,
    mut time_of_day: ResMut<TimeOfDay>,
    mut celestial: ResMut<CelestialState>,
    mut lighting: ResMut<DayNightLighting>,
) {
    runtime.active = true;
    runtime.last_announced_step = None;
    if !runtime.initialized {
        *time_of_day = initial_time_from_config(&config);
        runtime.initialized = true;
    }

    let phase = config.phase_boundaries.phase_at(time_of_day.cyclic_hour());
    let exposure_hint = config
        .lighting
        .exposure_ev100
        .sample(time_of_day.cyclic_hour());
    *celestial = solve_celestial_state(
        *time_of_day,
        &config.phase_boundaries,
        &config.celestial,
        exposure_hint,
    );
    *lighting = resolve_lighting(
        *time_of_day,
        celestial.as_ref(),
        &config.lighting,
        weather.as_ref(),
        &config.shadows,
    );
    celestial.suggested_exposure_ev100 = lighting.suggested_exposure_ev100;
    celestial.phase = phase;
    runtime.last_step = Some(TimeStep::idle(*time_of_day));
    runtime.lighting_initialized = true;
}

pub(crate) fn deactivate_runtime(
    mut commands: Commands,
    mut runtime: ResMut<DayNightRuntimeState>,
) {
    runtime.active = false;
    runtime.last_announced_step = None;

    if let Some(entity) = runtime.spawned_sun.take() {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = runtime.spawned_moon.take() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn runtime_is_active(runtime: Res<DayNightRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn ensure_managed_lights(
    mut commands: Commands,
    config: Res<DayNightConfig>,
    mut runtime: ResMut<DayNightRuntimeState>,
    suns: Query<Entity, With<Sun>>,
    moons: Query<Entity, With<Moon>>,
) {
    if !config.managed_lights.auto_spawn {
        return;
    }

    if suns.is_empty() {
        let entity = commands
            .spawn((
                Name::new("Managed Sun"),
                Sun,
                DirectionalLight {
                    illuminance: 0.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
            ))
            .id();
        runtime.spawned_sun = Some(entity);
    }

    if moons.is_empty() {
        let entity = commands
            .spawn((
                Name::new("Managed Moon"),
                Moon,
                DirectionalLight {
                    illuminance: 0.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
            ))
            .id();
        runtime.spawned_moon = Some(entity);
    }
}

pub(crate) fn advance_time(
    time: Res<Time>,
    mut config: ResMut<DayNightConfig>,
    mut time_of_day: ResMut<TimeOfDay>,
    mut runtime: ResMut<DayNightRuntimeState>,
) {
    let step = if let Some(request) = config.pending_override.take() {
        crate::apply_time_override(*time_of_day, request, config.wrap_mode)
    } else {
        crate::advance_continuous(
            *time_of_day,
            time.delta_secs(),
            config.seconds_per_hour,
            config.time_scale,
            config.paused,
            config.wrap_mode,
        )
    };

    if step.mode != TimeStepMode::Idle {
        *time_of_day = step.current;
    }

    runtime.last_step = Some(step);
}

pub(crate) fn resolve_celestial_state(
    config: Res<DayNightConfig>,
    time_of_day: Res<TimeOfDay>,
    lighting: Res<DayNightLighting>,
    mut celestial: ResMut<CelestialState>,
) {
    let exposure_hint = lighting.suggested_exposure_ev100;
    *celestial = solve_celestial_state(
        *time_of_day,
        &config.phase_boundaries,
        &config.celestial,
        exposure_hint,
    );
}

pub(crate) fn resolve_lighting_state(
    config: Res<DayNightConfig>,
    time: Res<Time>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<crate::WeatherModulation>,
    mut celestial: ResMut<CelestialState>,
    mut lighting: ResMut<DayNightLighting>,
    mut runtime: ResMut<DayNightRuntimeState>,
) {
    let target = resolve_lighting(
        *time_of_day,
        celestial.as_ref(),
        &config.lighting,
        weather.as_ref(),
        &config.shadows,
    );
    let step_mode = runtime
        .last_step
        .map(|step| step.mode)
        .unwrap_or(TimeStepMode::Idle);

    let resolved = if runtime.lighting_initialized {
        crate::lighting::smooth_lighting(
            lighting.as_ref(),
            &target,
            &config.smoothing,
            step_mode,
            time.delta_secs(),
        )
    } else {
        runtime.lighting_initialized = true;
        target
    };

    celestial.suggested_exposure_ev100 = resolved.suggested_exposure_ev100;
    *lighting = resolved;
}

pub(crate) fn detect_phase_transitions(
    config: Res<DayNightConfig>,
    mut runtime: ResMut<DayNightRuntimeState>,
    mut diagnostics: ResMut<DayNightDiagnostics>,
    mut dawn_started: MessageWriter<DawnStarted>,
    mut day_started: MessageWriter<DayStarted>,
    mut dusk_started: MessageWriter<DuskStarted>,
    mut night_started: MessageWriter<NightStarted>,
) {
    let Some(step) = runtime.last_step else {
        return;
    };

    diagnostics.last_step_mode = step.mode;

    if step.mode == TimeStepMode::Idle {
        return;
    }

    let current_step = (
        step.previous_absolute_hours(),
        step.current_absolute_hours(),
        step.mode,
    );
    if runtime.last_announced_step == Some(current_step) {
        return;
    }
    runtime.last_announced_step = Some(current_step);

    let phases = config.phase_boundaries.phases_started_between(
        step.previous_absolute_hours(),
        step.current_absolute_hours(),
    );

    for phase in phases {
        match phase {
            crate::DayPhase::Dawn => {
                let _ = dawn_started.write(DawnStarted);
            }
            crate::DayPhase::Day => {
                let _ = day_started.write(DayStarted);
            }
            crate::DayPhase::Dusk => {
                let _ = dusk_started.write(DuskStarted);
            }
            crate::DayPhase::Night => {
                let _ = night_started.write(NightStarted);
            }
        }

        diagnostics.last_phase_change = Some(phase);
        diagnostics.phase_message_count += 1;
        diagnostics.phase_history.push(phase);
        if diagnostics.phase_history.len() > 16 {
            diagnostics.phase_history.remove(0);
        }
    }
}

pub(crate) fn apply_managed_sun(
    config: Res<DayNightConfig>,
    celestial: Res<CelestialState>,
    lighting: Res<DayNightLighting>,
    mut diagnostics: ResMut<DayNightDiagnostics>,
    mut suns: Query<(&mut DirectionalLight, &mut Transform), (With<Sun>, Without<Moon>)>,
) {
    for (mut light, mut transform) in &mut suns {
        let mut wrote = false;
        wrote |= update_direction(
            &mut transform,
            celestial.sun_direction,
            config.write_thresholds.direction_dot_epsilon,
        );
        wrote |= update_color(
            &mut light.color,
            lighting.sun_color,
            config.write_thresholds.color_epsilon,
        );
        wrote |= update_scalar(
            &mut light.illuminance,
            lighting.sun_illuminance_lux,
            config.write_thresholds.illuminance_epsilon,
        );
        if light.shadows_enabled != lighting.sun_shadows_enabled {
            light.shadows_enabled = lighting.sun_shadows_enabled;
            wrote = true;
        }

        if wrote {
            diagnostics.sun_writes += 1;
        }
    }
}

pub(crate) fn apply_managed_moon(
    config: Res<DayNightConfig>,
    celestial: Res<CelestialState>,
    lighting: Res<DayNightLighting>,
    mut diagnostics: ResMut<DayNightDiagnostics>,
    mut moons: Query<(&mut DirectionalLight, &mut Transform), (With<Moon>, Without<Sun>)>,
) {
    for (mut light, mut transform) in &mut moons {
        let mut wrote = false;
        wrote |= update_direction(
            &mut transform,
            celestial.moon_direction,
            config.write_thresholds.direction_dot_epsilon,
        );
        wrote |= update_color(
            &mut light.color,
            lighting.moon_color,
            config.write_thresholds.color_epsilon,
        );
        wrote |= update_scalar(
            &mut light.illuminance,
            lighting.moon_illuminance_lux,
            config.write_thresholds.illuminance_epsilon,
        );
        if light.shadows_enabled != lighting.moon_shadows_enabled {
            light.shadows_enabled = lighting.moon_shadows_enabled;
            wrote = true;
        }

        if wrote {
            diagnostics.moon_writes += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_global_ambient_and_cameras(
    mut commands: Commands,
    config: Res<DayNightConfig>,
    lighting: Res<DayNightLighting>,
    mut diagnostics: ResMut<DayNightDiagnostics>,
    mut global_ambient: ResMut<GlobalAmbientLight>,
    mut atmosphere_cache: ResMut<AtmosphereAssetCache>,
    mut scattering_media: Option<ResMut<Assets<ScatteringMedium>>>,
    mut cameras: Query<
        (
            Entity,
            &DayNightCamera,
            Option<&DistanceFog>,
            Option<&mut VolumetricFog>,
            Option<&mut Exposure>,
            Option<&mut AtmosphereEnvironmentMapLight>,
            Option<&mut Atmosphere>,
            Option<&mut AtmosphereSettings>,
        ),
        With<Camera>,
    >,
) {
    if config.global_ambient.apply {
        let mut ambient_wrote = false;
        ambient_wrote |= update_color(
            &mut global_ambient.color,
            lighting.ambient_color,
            config.write_thresholds.color_epsilon,
        );
        ambient_wrote |= update_scalar(
            &mut global_ambient.brightness,
            lighting.ambient_brightness,
            config.write_thresholds.ambient_brightness_epsilon,
        );
        if ambient_wrote {
            diagnostics.ambient_writes += 1;
        }
    }

    let environment_map_intensity = (0.45 + lighting.ambient_brightness / 36.0)
        * config.atmosphere.environment_map_intensity_scale;

    for (
        entity,
        camera,
        distance_fog,
        volumetric_fog,
        exposure,
        environment_map,
        atmosphere,
        atmosphere_settings,
    ) in &mut cameras
    {
        if !camera.enabled {
            continue;
        }

        if camera.apply_distance_fog {
            if let Some(fog) = distance_fog {
                let directional_light_color = lighting.sun_color.with_alpha(
                    (0.10 + lighting.daylight_factor * 0.32 + lighting.twilight_factor * 0.22)
                        .clamp(0.0, 1.0),
                );
                let next_falloff = FogFalloff::from_visibility_colors(
                    lighting.fog_visibility,
                    lighting.fog_color.with_alpha(1.0),
                    lighting.ambient_color.with_alpha(1.0),
                );
                let should_update = !color_approx_eq(
                    fog.color,
                    lighting.fog_color,
                    config.write_thresholds.color_epsilon,
                ) || !color_approx_eq(
                    fog.directional_light_color,
                    directional_light_color,
                    config.write_thresholds.color_epsilon,
                ) || (fog.directional_light_exponent - 26.0).abs() > 1e-4
                    || !fog_falloff_approx_eq(
                        &fog.falloff,
                        &next_falloff,
                        config.write_thresholds.fog_visibility_epsilon,
                        config.write_thresholds.color_epsilon,
                    );

                if should_update {
                    commands.entity(entity).insert(DistanceFog {
                        color: lighting.fog_color,
                        directional_light_color,
                        directional_light_exponent: 26.0,
                        falloff: next_falloff,
                    });
                    diagnostics.fog_writes += 1;
                }
            } else if camera.insert_missing_components {
                commands
                    .entity(entity)
                    .insert(default_distance_fog(&lighting));
                diagnostics.fog_writes += 1;
            }
        }

        if camera.apply_volumetric_fog {
            if let Some(mut fog) = volumetric_fog {
                let mut wrote = false;
                wrote |= update_color(
                    &mut fog.ambient_color,
                    lighting.ambient_color,
                    config.write_thresholds.color_epsilon,
                );
                wrote |= update_scalar(
                    &mut fog.ambient_intensity,
                    lighting.volumetric_ambient_intensity,
                    config.write_thresholds.ambient_brightness_epsilon,
                );
                if wrote {
                    diagnostics.fog_writes += 1;
                }
            } else if camera.insert_missing_components {
                commands.entity(entity).insert(VolumetricFog {
                    ambient_color: lighting.ambient_color,
                    ambient_intensity: lighting.volumetric_ambient_intensity,
                    ..default()
                });
                diagnostics.fog_writes += 1;
            }
        }

        if camera.apply_exposure {
            if let Some(mut camera_exposure) = exposure {
                if update_scalar(
                    &mut camera_exposure.ev100,
                    lighting.suggested_exposure_ev100,
                    config.write_thresholds.exposure_epsilon,
                ) {
                    diagnostics.exposure_writes += 1;
                }
            } else if camera.insert_missing_components {
                commands.entity(entity).insert(Exposure {
                    ev100: lighting.suggested_exposure_ev100,
                });
                diagnostics.exposure_writes += 1;
            }
        }

        if camera.apply_environment_map_light {
            if let Some(mut atmosphere_light) = environment_map {
                if update_scalar(
                    &mut atmosphere_light.intensity,
                    environment_map_intensity,
                    config.write_thresholds.ambient_brightness_epsilon,
                ) {
                    diagnostics.environment_map_writes += 1;
                }
            } else if camera.insert_missing_components {
                commands
                    .entity(entity)
                    .insert(AtmosphereEnvironmentMapLight {
                        intensity: environment_map_intensity,
                        ..default()
                    });
                diagnostics.environment_map_writes += 1;
            }
        }

        if camera.ensure_atmosphere {
            let atmosphere_handle = scattering_media.as_mut().map(|media| {
                earthlike_medium_handle(
                    atmosphere_cache.as_mut(),
                    media.as_mut(),
                    config.atmosphere.density_multiplier,
                )
            });

            if let Some(mut settings_component) = atmosphere_settings {
                update_scalar(
                    &mut settings_component.scene_units_to_m,
                    config.atmosphere.scene_units_to_m,
                    config.write_thresholds.ambient_brightness_epsilon,
                );
            } else if camera.insert_missing_components {
                commands.entity(entity).insert(AtmosphereSettings {
                    scene_units_to_m: config.atmosphere.scene_units_to_m,
                    ..default()
                });
            }

            if let Some(handle) = atmosphere_handle {
                if let Some(mut atmosphere_component) = atmosphere {
                    if atmosphere_component.medium != handle {
                        atmosphere_component.medium = handle;
                    }
                } else if camera.insert_missing_components {
                    commands
                        .entity(entity)
                        .insert(Atmosphere::earthlike(handle));
                }
            }
        }
    }
}

pub(crate) fn update_time_reactive(
    mut commands: Commands,
    time_of_day: Res<TimeOfDay>,
    query: Query<(Entity, &crate::TimeReactive, Has<crate::TimeActive>)>,
) {
    let hour = time_of_day.cyclic_hour();
    for (entity, reactive, currently_active) in &query {
        let should_be_active = reactive.is_active_at(hour);
        if should_be_active && !currently_active {
            commands.entity(entity).insert(crate::TimeActive);
        } else if !should_be_active && currently_active {
            commands.entity(entity).remove::<crate::TimeActive>();
        }
    }
}

pub(crate) fn publish_diagnostics(
    time_of_day: Res<TimeOfDay>,
    celestial: Res<CelestialState>,
    runtime: Res<DayNightRuntimeState>,
    mut diagnostics: ResMut<DayNightDiagnostics>,
) {
    diagnostics.current_time = time_of_day.hour;
    diagnostics.elapsed_days = time_of_day.elapsed_days;
    diagnostics.current_phase = celestial.phase;
    diagnostics.last_step_mode = runtime
        .last_step
        .map(|step| step.mode)
        .unwrap_or(TimeStepMode::Idle);
}

fn initial_time_from_config(config: &DayNightConfig) -> TimeOfDay {
    let hour = match config.wrap_mode {
        crate::TimeWrapMode::Loop => crate::time::normalize_hour(config.initial_time),
        crate::TimeWrapMode::Clamp => crate::time::clamp_hour(config.initial_time),
    };
    TimeOfDay::with_days(hour, 0)
}

fn earthlike_medium_handle(
    cache: &mut AtmosphereAssetCache,
    media: &mut Assets<ScatteringMedium>,
    density_multiplier: f32,
) -> Handle<ScatteringMedium> {
    let clamped_density = density_multiplier.max(0.01);
    let needs_refresh = cache.earthlike.is_none()
        || cache
            .density_multiplier
            .map(|current| (current - clamped_density).abs() > 1e-4)
            .unwrap_or(true);

    if needs_refresh {
        let handle = media
            .add(ScatteringMedium::earthlike(256, 256).with_density_multiplier(clamped_density));
        cache.earthlike = Some(handle.clone());
        cache.density_multiplier = Some(clamped_density);
        return handle;
    }

    cache
        .earthlike
        .clone()
        .expect("earthlike handle should be cached when no refresh is needed")
}

fn default_distance_fog(lighting: &DayNightLighting) -> DistanceFog {
    DistanceFog {
        color: lighting.fog_color,
        directional_light_color: lighting.sun_color.with_alpha(
            (0.10 + lighting.daylight_factor * 0.32 + lighting.twilight_factor * 0.22)
                .clamp(0.0, 1.0),
        ),
        directional_light_exponent: 26.0,
        falloff: FogFalloff::from_visibility_colors(
            lighting.fog_visibility,
            lighting.fog_color.with_alpha(1.0),
            lighting.ambient_color.with_alpha(1.0),
        ),
    }
}

fn update_direction(transform: &mut Transform, direction: Vec3, epsilon: f32) -> bool {
    let target = direction.normalize_or_zero();
    if target == Vec3::ZERO {
        return false;
    }

    let current = transform.rotation.mul_vec3(Vec3::NEG_Z).normalize_or_zero();
    if current.dot(target) >= 1.0 - epsilon {
        return false;
    }

    transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, target);
    true
}

fn update_color(current: &mut Color, next: Color, epsilon: f32) -> bool {
    if color_approx_eq(*current, next, epsilon) {
        return false;
    }
    *current = next;
    true
}

fn update_scalar(current: &mut f32, next: f32, epsilon: f32) -> bool {
    if (*current - next).abs() <= epsilon {
        return false;
    }
    *current = next;
    true
}

fn fog_falloff_approx_eq(
    left: &FogFalloff,
    right: &FogFalloff,
    scalar_epsilon: f32,
    vector_epsilon: f32,
) -> bool {
    match (left, right) {
        (
            FogFalloff::Linear {
                start: left_start,
                end: left_end,
            },
            FogFalloff::Linear {
                start: right_start,
                end: right_end,
            },
        ) => {
            (left_start - right_start).abs() <= scalar_epsilon
                && (left_end - right_end).abs() <= scalar_epsilon
        }
        (
            FogFalloff::Exponential {
                density: left_density,
            },
            FogFalloff::Exponential {
                density: right_density,
            },
        )
        | (
            FogFalloff::ExponentialSquared {
                density: left_density,
            },
            FogFalloff::ExponentialSquared {
                density: right_density,
            },
        ) => (left_density - right_density).abs() <= scalar_epsilon,
        (
            FogFalloff::Atmospheric {
                extinction: left_extinction,
                inscattering: left_inscattering,
            },
            FogFalloff::Atmospheric {
                extinction: right_extinction,
                inscattering: right_inscattering,
            },
        ) => {
            vec3_approx_eq(*left_extinction, *right_extinction, vector_epsilon)
                && vec3_approx_eq(*left_inscattering, *right_inscattering, vector_epsilon)
        }
        _ => false,
    }
}

fn vec3_approx_eq(left: Vec3, right: Vec3, epsilon: f32) -> bool {
    left.distance_squared(right) <= epsilon * epsilon
}

fn color_approx_eq(left: Color, right: Color, epsilon: f32) -> bool {
    let left = LinearRgba::from(left);
    let right = LinearRgba::from(right);
    let delta = (left.red - right.red).abs()
        + (left.green - right.green).abs()
        + (left.blue - right.blue).abs()
        + (left.alpha - right.alpha).abs();
    delta <= epsilon
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
