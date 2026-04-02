use bevy::prelude::*;

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
