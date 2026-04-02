use bevy::prelude::*;
use saddle_world_day_night::{
    CelestialState, DayNightCamera, DayNightDiagnostics, DayNightLighting, Moon, Sun, TimeOfDay,
};

#[derive(Component)]
pub struct ShowcaseSpinner {
    pub axis: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct ShowcaseOverlay;

pub fn spawn_outdoor_showcase(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    camera: DayNightCamera,
    with_overlay: bool,
) -> Entity {
    commands.spawn((
        Name::new("Showcase Sun"),
        Sun,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Showcase Moon"),
        Moon,
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default(),
    ));
    let camera_entity = commands
        .spawn((
            Name::new("Showcase Camera"),
            Camera3d::default(),
            camera,
            Transform::from_xyz(-9.0, 6.0, -12.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        ))
        .id();

    commands.spawn((
        Name::new("Showcase Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.21, 0.24),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    let palette = [
        (
            Vec3::new(-8.0, 1.3, -8.0),
            Vec3::new(1.4, 2.6, 1.4),
            Color::srgb(0.27, 0.42, 0.68),
        ),
        (
            Vec3::new(-2.5, 1.7, -3.0),
            Vec3::new(2.4, 3.4, 2.0),
            Color::srgb(0.74, 0.39, 0.28),
        ),
        (
            Vec3::new(3.2, 1.1, 2.2),
            Vec3::new(1.8, 2.2, 1.8),
            Color::srgb(0.28, 0.64, 0.46),
        ),
        (
            Vec3::new(9.0, 2.1, -1.4),
            Vec3::new(2.2, 4.2, 2.2),
            Color::srgb(0.78, 0.68, 0.24),
        ),
        (
            Vec3::new(0.0, 0.7, 8.4),
            Vec3::new(5.0, 1.4, 2.8),
            Color::srgb(0.38, 0.28, 0.22),
        ),
    ];

    for (index, (translation, scale, color)) in palette.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Showcase Prop {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.05,
                perceptual_roughness: 0.42,
                ..default()
            })),
            Transform::from_translation(translation),
            ShowcaseSpinner {
                axis: Vec3::new(0.2 + index as f32 * 0.07, 1.0, 0.18).normalize(),
                speed: 0.08 + index as f32 * 0.03,
            },
        ));
    }

    if with_overlay {
        commands.spawn((
            Name::new("Showcase Overlay"),
            ShowcaseOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(420.0),
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.72)),
            Text::default(),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }

    camera_entity
}

pub fn spin_showcase(time: Res<Time>, mut query: Query<(&ShowcaseSpinner, &mut Transform)>) {
    for (spinner, mut transform) in &mut query {
        transform.rotate(Quat::from_axis_angle(
            spinner.axis,
            spinner.speed * time.delta_secs(),
        ));
    }
}

pub fn update_overlay(
    time_of_day: Res<TimeOfDay>,
    celestial: Res<CelestialState>,
    lighting: Res<DayNightLighting>,
    diagnostics: Res<DayNightDiagnostics>,
    mut overlay: Query<&mut Text, With<ShowcaseOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    text.0 = format!(
        "Time {:05.2}  Day {}\nPhase {:?}\nSun elevation {:>6.2}°  Moon elevation {:>6.2}°\nSun lux {:>8.1}  Moon lux {:>6.3}\nAmbient {:>5.2}  Fog vis {:>6.1}\nExposure EV100 {:>4.2}  Stars {:>4.2}\nPhase messages {}",
        time_of_day.hour,
        time_of_day.elapsed_days,
        celestial.phase,
        celestial.sun_elevation_degrees,
        celestial.moon_elevation_degrees,
        lighting.sun_illuminance_lux,
        lighting.moon_illuminance_lux,
        lighting.ambient_brightness,
        lighting.fog_visibility,
        lighting.suggested_exposure_ev100,
        lighting.star_visibility,
        diagnostics.phase_message_count,
    );
}
