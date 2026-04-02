use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy)]
pub struct DawnStarted;

#[derive(Message, Debug, Clone, Copy)]
pub struct DayStarted;

#[derive(Message, Debug, Clone, Copy)]
pub struct DuskStarted;

#[derive(Message, Debug, Clone, Copy)]
pub struct NightStarted;
