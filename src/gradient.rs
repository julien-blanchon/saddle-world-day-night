use bevy::prelude::*;

use crate::time::{DAY_LENGTH_HOURS, normalize_hour};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ScalarKeyframe {
    pub hour: f32,
    pub value: f32,
}

impl ScalarKeyframe {
    pub fn new(hour: f32, value: f32) -> Self {
        Self { hour, value }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct ScalarGradient {
    pub keys: Vec<ScalarKeyframe>,
}

impl ScalarGradient {
    pub fn constant(value: f32) -> Self {
        Self {
            keys: vec![ScalarKeyframe::new(0.0, value)],
        }
    }

    pub fn new(mut keys: Vec<ScalarKeyframe>) -> Self {
        keys.sort_by(|a, b| a.hour.total_cmp(&b.hour));
        Self { keys }
    }

    pub fn sample(&self, hour: f32) -> f32 {
        let hour = normalize_hour(hour);
        match self.keys.as_slice() {
            [] => 0.0,
            [only] => only.value,
            keys => {
                let (left, right, t) = enclosing_keyframes_scalar(keys, hour);
                left.value + (right.value - left.value) * t
            }
        }
    }
}

impl Default for ScalarGradient {
    fn default() -> Self {
        Self::constant(1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ColorKeyframe {
    pub hour: f32,
    pub color: Color,
}

impl ColorKeyframe {
    pub fn new(hour: f32, color: Color) -> Self {
        Self { hour, color }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct ColorGradient {
    pub keys: Vec<ColorKeyframe>,
}

impl ColorGradient {
    pub fn constant(color: Color) -> Self {
        Self {
            keys: vec![ColorKeyframe::new(0.0, color)],
        }
    }

    pub fn new(mut keys: Vec<ColorKeyframe>) -> Self {
        keys.sort_by(|a, b| a.hour.total_cmp(&b.hour));
        Self { keys }
    }

    pub fn sample(&self, hour: f32) -> Color {
        let hour = normalize_hour(hour);
        match self.keys.as_slice() {
            [] => Color::WHITE,
            [only] => only.color,
            keys => {
                let (left, right, t) = enclosing_keyframes_color(keys, hour);
                if t <= 1e-6 {
                    left.color
                } else if (1.0 - t) <= 1e-6 {
                    right.color
                } else {
                    mix_color(left.color, right.color, t)
                }
            }
        }
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self::constant(Color::WHITE)
    }
}

pub fn mix_color(left: Color, right: Color, t: f32) -> Color {
    let left = LinearRgba::from(left);
    let right = LinearRgba::from(right);
    Color::LinearRgba(LinearRgba {
        red: left.red + (right.red - left.red) * t,
        green: left.green + (right.green - left.green) * t,
        blue: left.blue + (right.blue - left.blue) * t,
        alpha: left.alpha + (right.alpha - left.alpha) * t,
    })
}

fn enclosing_keyframes_scalar(
    keys: &[ScalarKeyframe],
    hour: f32,
) -> (ScalarKeyframe, ScalarKeyframe, f32) {
    for window in keys.windows(2) {
        let left = window[0];
        let right = window[1];
        if hour >= left.hour && hour <= right.hour {
            let span = (right.hour - left.hour).max(1e-4);
            return (left, right, (hour - left.hour) / span);
        }
    }

    let left = *keys.last().expect("at least one keyframe");
    let right = keys[0];
    let wrapped_hour = if hour < right.hour {
        hour + DAY_LENGTH_HOURS
    } else {
        hour
    };
    let span = (right.hour + DAY_LENGTH_HOURS - left.hour).max(1e-4);
    (left, right, (wrapped_hour - left.hour) / span)
}

fn enclosing_keyframes_color(
    keys: &[ColorKeyframe],
    hour: f32,
) -> (ColorKeyframe, ColorKeyframe, f32) {
    for window in keys.windows(2) {
        let left = window[0];
        let right = window[1];
        if hour >= left.hour && hour <= right.hour {
            let span = (right.hour - left.hour).max(1e-4);
            return (left, right, (hour - left.hour) / span);
        }
    }

    let left = *keys.last().expect("at least one keyframe");
    let right = keys[0];
    let wrapped_hour = if hour < right.hour {
        hour + DAY_LENGTH_HOURS
    } else {
        hour
    };
    let span = (right.hour + DAY_LENGTH_HOURS - left.hour).max(1e-4);
    (left, right, (wrapped_hour - left.hour) / span)
}

#[cfg(test)]
#[path = "gradient_tests.rs"]
mod tests;
