use bevy::prelude::*;

pub const DAY_LENGTH_HOURS: f32 = 24.0;
const DAY_LENGTH_F64: f64 = DAY_LENGTH_HOURS as f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum TimeWrapMode {
    #[default]
    Loop,
    Clamp,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Resource, Default)]
pub struct TimeOfDay {
    pub hour: f32,
    pub elapsed_days: u32,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self::new(12.0)
    }
}

impl TimeOfDay {
    pub fn new(hour: f32) -> Self {
        Self {
            hour: clamp_hour(hour),
            elapsed_days: 0,
        }
    }

    pub fn with_days(hour: f32, elapsed_days: u32) -> Self {
        Self {
            hour: clamp_hour(hour),
            elapsed_days,
        }
    }

    pub fn cyclic_hour(self) -> f32 {
        normalize_hour(self.hour)
    }

    pub fn total_hours(self) -> f64 {
        f64::from(self.elapsed_days) * DAY_LENGTH_F64 + f64::from(self.hour)
    }

    pub fn set_hour(&mut self, hour: f32) {
        self.hour = clamp_hour(hour);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum TimeOverride {
    Scrub { hour: f32 },
    AdvanceHours { hours: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum TimeStepMode {
    Idle,
    Continuous,
    Scrub,
    AdvanceJump,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct TimeStep {
    pub previous: TimeOfDay,
    pub current: TimeOfDay,
    pub mode: TimeStepMode,
    pub delta_hours: f32,
}

impl TimeStep {
    pub fn idle(time: TimeOfDay) -> Self {
        Self {
            previous: time,
            current: time,
            mode: TimeStepMode::Idle,
            delta_hours: 0.0,
        }
    }

    pub fn current_absolute_hours(self) -> f64 {
        self.current.total_hours()
    }

    pub fn previous_absolute_hours(self) -> f64 {
        self.previous.total_hours()
    }
}

pub fn normalize_hour(hour: f32) -> f32 {
    let wrapped = hour.rem_euclid(DAY_LENGTH_HOURS);
    if (DAY_LENGTH_HOURS - wrapped).abs() <= 1e-4 || wrapped >= DAY_LENGTH_HOURS {
        0.0
    } else {
        wrapped
    }
}

pub fn clamp_hour(hour: f32) -> f32 {
    hour.clamp(0.0, DAY_LENGTH_HOURS)
}

pub fn advance_continuous(
    time: TimeOfDay,
    delta_seconds: f32,
    seconds_per_hour: f32,
    time_scale: f32,
    paused: bool,
    wrap_mode: TimeWrapMode,
) -> TimeStep {
    if paused || delta_seconds == 0.0 || time_scale == 0.0 || seconds_per_hour <= 0.0 {
        return TimeStep::idle(time);
    }

    let delta_hours = delta_seconds / seconds_per_hour * time_scale;
    advance_by_hours(time, delta_hours, wrap_mode, TimeStepMode::Continuous)
}

pub fn apply_time_override(
    time: TimeOfDay,
    request: TimeOverride,
    wrap_mode: TimeWrapMode,
) -> TimeStep {
    match request {
        TimeOverride::Scrub { hour } => {
            let mut current = time;
            current.hour = match wrap_mode {
                TimeWrapMode::Loop => normalize_hour(hour),
                TimeWrapMode::Clamp => clamp_hour(hour),
            };
            TimeStep {
                previous: time,
                current,
                mode: TimeStepMode::Scrub,
                delta_hours: current.hour - time.hour,
            }
        }
        TimeOverride::AdvanceHours { hours } => {
            advance_by_hours(time, hours, wrap_mode, TimeStepMode::AdvanceJump)
        }
    }
}

pub fn advance_by_hours(
    time: TimeOfDay,
    delta_hours: f32,
    wrap_mode: TimeWrapMode,
    mode: TimeStepMode,
) -> TimeStep {
    if delta_hours == 0.0 {
        return TimeStep::idle(time);
    }

    match wrap_mode {
        TimeWrapMode::Loop => advance_looping(time, delta_hours, mode),
        TimeWrapMode::Clamp => advance_clamped(time, delta_hours, mode),
    }
}

fn advance_looping(time: TimeOfDay, delta_hours: f32, mode: TimeStepMode) -> TimeStep {
    let total = time.total_hours() + f64::from(delta_hours);
    let clamped_total = total.max(0.0);
    let elapsed_days = (clamped_total / DAY_LENGTH_F64).floor() as u32;
    let mut hour = (clamped_total - f64::from(elapsed_days) * DAY_LENGTH_F64) as f32;
    if hour >= DAY_LENGTH_HOURS - 1e-4 {
        hour = 0.0;
    }

    TimeStep {
        previous: time,
        current: TimeOfDay { hour, elapsed_days },
        mode,
        delta_hours,
    }
}

fn advance_clamped(time: TimeOfDay, delta_hours: f32, mode: TimeStepMode) -> TimeStep {
    let hour = (time.hour + delta_hours).clamp(0.0, DAY_LENGTH_HOURS);
    TimeStep {
        previous: time,
        current: TimeOfDay {
            hour,
            elapsed_days: time.elapsed_days,
        },
        mode,
        delta_hours,
    }
}

#[cfg(test)]
#[path = "time_tests.rs"]
mod tests;
