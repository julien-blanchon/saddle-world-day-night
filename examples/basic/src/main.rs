use saddle_world_day_night_example_support as support;

use bevy::prelude::*;
use saddle_world_day_night::{DayNightConfig, DayNightPlugin};

fn main() {
    let config = DayNightConfig {
        initial_time: 7.5,
        seconds_per_hour: 2.0,
        ..default()
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night basic".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &config);
    app.add_plugins(DayNightPlugin::default().with_config(config));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (support::spin_showcase, support::update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let _ = support::spawn_outdoor_showcase(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        saddle_world_day_night::DayNightCamera::default(),
        true,
    );
    support::spawn_instructions(
        &mut commands,
        "Basic Day/Night Demo\n\
         Use the pane (top-right) to pause, scrub time, or adjust cycle speed.\n\
         Watch the sun, shadows, ambient light, and fog change over the day.",
    );
}
