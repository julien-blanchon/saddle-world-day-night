use saddle_world_day_night_example_support as support;

use bevy::prelude::*;
use saddle_world_day_night::{
    DayNightCamera, DayNightConfig, DayNightPlugin, DayNightSystems, TimeActive, TimeReactive,
};

/// Marker for lamp point lights — brightness is driven by TimeActive presence.
#[derive(Component)]
struct LampLight;

fn main() {
    let config = DayNightConfig {
        initial_time: 17.5,
        seconds_per_hour: 1.5,
        ..default()
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night street_lights".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &config);
    app.add_plugins(DayNightPlugin::default().with_config(config));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::update_overlay,
            drive_lamp_lights.after(DayNightSystems::UpdateTimeReactive),
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Name::new("Street Camera"),
        Camera3d::default(),
        DayNightCamera::default(),
        Transform::from_xyz(-18.0, 10.0, 22.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));

    // Sun & Moon (managed by the plugin via auto_spawn)

    // Ground
    commands.spawn((
        Name::new("Street Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.16),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    // Road
    commands.spawn((
        Name::new("Road"),
        Mesh3d(meshes.add(Cuboid::new(60.0, 0.02, 6.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.12, 0.13),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, 0.0),
    ));

    // Buildings along one side
    let building_data = [
        (
            Vec3::new(-12.0, 3.0, -8.0),
            Vec3::new(5.0, 6.0, 4.0),
            Color::srgb(0.45, 0.38, 0.32),
        ),
        (
            Vec3::new(-4.0, 4.5, -9.0),
            Vec3::new(6.0, 9.0, 5.0),
            Color::srgb(0.52, 0.46, 0.40),
        ),
        (
            Vec3::new(5.0, 3.5, -7.5),
            Vec3::new(7.0, 7.0, 4.5),
            Color::srgb(0.40, 0.42, 0.48),
        ),
        (
            Vec3::new(14.0, 2.5, -8.0),
            Vec3::new(5.5, 5.0, 4.0),
            Color::srgb(0.48, 0.36, 0.30),
        ),
    ];

    for (index, (pos, size, color)) in building_data.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Building {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                metallic: 0.02,
                perceptual_roughness: 0.65,
                ..default()
            })),
            Transform::from_translation(*pos),
        ));
    }

    // Buildings along the other side
    let building_data_far = [
        (
            Vec3::new(-10.0, 2.5, 8.0),
            Vec3::new(4.5, 5.0, 3.5),
            Color::srgb(0.38, 0.40, 0.45),
        ),
        (
            Vec3::new(0.0, 3.0, 9.0),
            Vec3::new(5.0, 6.0, 4.0),
            Color::srgb(0.50, 0.42, 0.35),
        ),
        (
            Vec3::new(10.0, 4.0, 8.5),
            Vec3::new(6.0, 8.0, 5.0),
            Color::srgb(0.44, 0.38, 0.42),
        ),
    ];

    for (index, (pos, size, color)) in building_data_far.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Far Building {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                metallic: 0.02,
                perceptual_roughness: 0.65,
                ..default()
            })),
            Transform::from_translation(*pos),
        ));
    }

    // Street lamp material
    let pole_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.20, 0.22),
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Street lamps along the road
    let lamp_positions = [-15.0, -7.5, 0.0, 7.5, 15.0];
    for (index, &x) in lamp_positions.iter().enumerate() {
        let pole_height = 4.5;

        // Pole
        commands.spawn((
            Name::new(format!("Lamp Pole {}", index + 1)),
            Mesh3d(meshes.add(Cylinder::new(0.08, pole_height))),
            MeshMaterial3d(pole_material.clone()),
            Transform::from_xyz(x, pole_height / 2.0, 4.8),
        ));

        // Lamp head (small sphere)
        commands.spawn((
            Name::new(format!("Lamp Head {}", index + 1)),
            Mesh3d(meshes.add(Sphere::new(0.22).mesh().uv(16, 12))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.88, 0.65),
                emissive: LinearRgba::new(0.0, 0.0, 0.0, 1.0),
                ..default()
            })),
            Transform::from_xyz(x, pole_height + 0.1, 4.8),
        ));

        // Point light (night-active via TimeReactive)
        commands.spawn((
            Name::new(format!("Lamp Light {}", index + 1)),
            LampLight,
            PointLight {
                color: Color::srgb(1.0, 0.88, 0.62),
                intensity: 0.0,
                range: 14.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(x, pole_height + 0.15, 4.8),
            TimeReactive::night_active(),
        ));
    }

    // Overlay
    commands.spawn((
        Name::new("Street Overlay"),
        support::ShowcaseOverlay,
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

    // Instructions overlay
    commands.spawn((
        Name::new("Instructions"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.60)),
        Text::new(
            "Street Lights Demo\n\
             Lamps use TimeReactive to turn on at dusk (19:00) and off at dawn (6:00).\n\
             Use the pane controls (top-right) to pause, scrub time, or adjust speed.",
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.85, 0.9)),
    ));
}

/// Smoothly drives lamp point light intensity based on TimeActive presence.
fn drive_lamp_lights(
    time: Res<Time>,
    mut lamps: Query<(&mut PointLight, Has<TimeActive>), With<LampLight>>,
) {
    let target_on = 80_000.0_f32;
    let speed = 3.0; // transition speed (seconds⁻¹)

    for (mut light, is_active) in &mut lamps {
        let target = if is_active { target_on } else { 0.0 };
        let current = light.intensity;
        let step = (target - current) * (1.0 - (-speed * time.delta_secs()).exp());
        let next = current + step;
        // Snap to zero when close enough to avoid flickering
        let next = if next < 50.0 && target == 0.0 {
            0.0
        } else {
            next
        };
        if (light.intensity - next).abs() > 1.0 {
            light.intensity = next;
        }
    }
}
