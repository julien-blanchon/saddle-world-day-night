use bevy::prelude::*;

use super::{ColorGradient, ColorKeyframe, ScalarGradient, ScalarKeyframe};

#[test]
fn scalar_gradient_interpolates_between_keys() {
    let gradient = ScalarGradient::new(vec![
        ScalarKeyframe::new(0.0, 0.0),
        ScalarKeyframe::new(12.0, 1.0),
    ]);

    assert!((gradient.sample(6.0) - 0.5).abs() < 1e-4);
}

#[test]
fn scalar_gradient_wraps_smoothly_across_midnight() {
    let gradient = ScalarGradient::new(vec![
        ScalarKeyframe::new(18.0, 1.0),
        ScalarKeyframe::new(6.0, 0.0),
        ScalarKeyframe::new(12.0, 0.5),
    ]);

    let sample = gradient.sample(23.0);
    assert!(sample.is_finite());
    assert!(sample > 0.0);
}

#[test]
fn color_gradient_hits_exact_keyframe_value() {
    let color = Color::srgb(0.8, 0.3, 0.2);
    let gradient = ColorGradient::new(vec![
        ColorKeyframe::new(0.0, Color::BLACK),
        ColorKeyframe::new(6.0, color),
        ColorKeyframe::new(12.0, Color::WHITE),
    ]);

    assert_eq!(gradient.sample(6.0), color);
}

#[test]
fn color_gradient_wraps_without_nan() {
    let gradient = ColorGradient::new(vec![
        ColorKeyframe::new(18.0, Color::srgb(0.2, 0.2, 0.6)),
        ColorKeyframe::new(6.0, Color::srgb(0.9, 0.7, 0.4)),
        ColorKeyframe::new(12.0, Color::WHITE),
    ]);

    let color = gradient.sample(23.5).to_linear();
    assert!(color.red.is_finite());
    assert!(color.green.is_finite());
    assert!(color.blue.is_finite());
}
