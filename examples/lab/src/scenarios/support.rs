use bevy::prelude::*;
use saddle_bevy_e2e::action::Action;

use saddle_world_day_night::{DayNightLighting, TimeOfDay};

use crate::LabOverlay;

pub(super) fn entity_by_name<T: Component>(world: &mut World, target_name: &str) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, &Name), With<T>>();
    query
        .iter(world)
        .find_map(|(entity, name)| (name.as_str() == target_name).then_some(entity))
}

pub(super) fn overlay_text(world: &mut World) -> Option<String> {
    let mut query = world.query_filtered::<&Text, With<LabOverlay>>();
    query.iter(world).next().map(|text| text.0.clone())
}

pub(super) fn time_of_day(world: &World) -> Option<TimeOfDay> {
    world.get_resource::<TimeOfDay>().copied()
}

pub(super) fn lighting(world: &World) -> DayNightLighting {
    world
        .get_resource::<DayNightLighting>()
        .cloned()
        .expect("DayNightLighting resource should exist")
}

pub(super) fn pause_and_scrub(hour: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        {
            let mut pane =
                world.resource_mut::<saddle_world_day_night_example_support::DayNightDemoPane>();
            pane.paused = true;
            pane.time_hours = hour;
        }
        {
            let mut config = world.resource_mut::<saddle_world_day_night::DayNightConfig>();
            config.paused = true;
            config.pending_override = None;
        }
        world
            .resource_mut::<saddle_world_day_night::TimeOfDay>()
            .set_hour(hour);
    }))
}
