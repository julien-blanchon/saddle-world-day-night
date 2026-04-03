use saddle_world_day_night_example_support as support;

use bevy::prelude::*;
use saddle_world_day_night::{DayNightConfig, DayNightPlugin, LightingProfile};

fn main() {
    let config = DayNightConfig::default()
        .fixed_time(18.35)
        .with_profile(LightingProfile::stylized_saturated());
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night fixed_time".into(),
            resolution: (1440, 810).into(),
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
}
