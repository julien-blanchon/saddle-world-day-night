use saddle_world_day_night_example_support as support;

use bevy::{
    camera::Exposure, core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom,
    prelude::*,
};
use saddle_world_day_night::{DayNightCamera, DayNightConfig, DayNightPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night atmosphere".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(DayNightPlugin::default().with_config(DayNightConfig {
        initial_time: 6.0,
        seconds_per_hour: 1.4,
        ..default()
    }));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (support::spin_showcase, support::update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = support::spawn_outdoor_showcase(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        DayNightCamera {
            ensure_atmosphere: true,
            ..default()
        },
        true,
    );
    commands.entity(camera).insert((
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
    ));
}
