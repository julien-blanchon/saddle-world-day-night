use saddle_world_day_night_example_support as support;

use bevy::prelude::*;
use saddle_world_day_night::{
    CelestialModel, DayNightConfig, DayNightPlugin, DayNightSystems, SeasonSettings,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "day_night latitude".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(DayNightPlugin::default().with_config(DayNightConfig {
        initial_time: 4.5,
        seconds_per_hour: 1.0,
        celestial: saddle_world_day_night::CelestialSettings {
            model: CelestialModel::LatitudeAware {
                latitude_degrees: 62.0,
                season: SeasonSettings {
                    season_progress: 0.25,
                    ..default()
                },
            },
            ..default()
        },
        ..default()
    }));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::spin_showcase,
            support::update_overlay.after(DayNightSystems::ApplyLighting),
        ),
    );
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
