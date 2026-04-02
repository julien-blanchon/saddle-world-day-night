use bevy::prelude::*;

use crate::time::{DAY_LENGTH_HOURS, normalize_hour};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum DayPhase {
    Dawn,
    Day,
    Dusk,
    Night,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct DayPhaseBoundaries {
    pub dawn_starts: f32,
    pub day_starts: f32,
    pub dusk_starts: f32,
    pub night_starts: f32,
}

impl Default for DayPhaseBoundaries {
    fn default() -> Self {
        Self {
            dawn_starts: 5.5,
            day_starts: 7.0,
            dusk_starts: 18.0,
            night_starts: 19.5,
        }
    }
}

impl DayPhaseBoundaries {
    pub fn is_valid(&self) -> bool {
        self.dawn_starts >= 0.0
            && self.day_starts > self.dawn_starts
            && self.dusk_starts > self.day_starts
            && self.night_starts > self.dusk_starts
            && self.night_starts < DAY_LENGTH_HOURS
    }

    pub fn validated_or_default(self) -> Self {
        if self.is_valid() {
            self
        } else {
            Self::default()
        }
    }

    pub fn phase_at(&self, hour: f32) -> DayPhase {
        let hour = normalize_hour(hour);
        let boundaries = self.validated_or_default();
        if hour >= boundaries.night_starts || hour < boundaries.dawn_starts {
            DayPhase::Night
        } else if hour >= boundaries.dusk_starts {
            DayPhase::Dusk
        } else if hour >= boundaries.day_starts {
            DayPhase::Day
        } else {
            DayPhase::Dawn
        }
    }

    pub fn start_hour(&self, phase: DayPhase) -> f32 {
        let boundaries = self.validated_or_default();
        match phase {
            DayPhase::Dawn => boundaries.dawn_starts,
            DayPhase::Day => boundaries.day_starts,
            DayPhase::Dusk => boundaries.dusk_starts,
            DayPhase::Night => boundaries.night_starts,
        }
    }

    pub fn next_start(&self, phase: DayPhase, from_hour: f32) -> f32 {
        let from_hour = normalize_hour(from_hour);
        let start = self.start_hour(phase);
        if start > from_hour {
            start
        } else {
            start + DAY_LENGTH_HOURS
        }
    }

    pub fn hours_until_phase(&self, phase: DayPhase, from_hour: f32) -> f32 {
        self.next_start(phase, from_hour) - normalize_hour(from_hour)
    }

    pub fn phases_started_between(
        &self,
        previous_absolute_hours: f64,
        current_absolute_hours: f64,
    ) -> Vec<DayPhase> {
        if current_absolute_hours <= previous_absolute_hours {
            return Vec::new();
        }

        let boundaries = self.validated_or_default();
        let starts = [
            (f64::from(boundaries.dawn_starts), DayPhase::Dawn),
            (f64::from(boundaries.day_starts), DayPhase::Day),
            (f64::from(boundaries.dusk_starts), DayPhase::Dusk),
            (f64::from(boundaries.night_starts), DayPhase::Night),
        ];

        let start_day = (previous_absolute_hours / 24.0).floor() as i64;
        let end_day = (current_absolute_hours / 24.0).ceil() as i64;
        let mut phases = Vec::new();

        for day in start_day..=end_day {
            for (hour, phase) in starts {
                let absolute_hour = f64::from(day as i32) * 24.0 + hour;
                if absolute_hour > previous_absolute_hours + 1e-6
                    && absolute_hour <= current_absolute_hours + 1e-6
                {
                    phases.push(phase);
                }
            }
        }

        phases
    }
}

#[cfg(test)]
#[path = "phase_tests.rs"]
mod tests;
