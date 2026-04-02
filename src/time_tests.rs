use super::{
    DAY_LENGTH_HOURS, TimeOfDay, TimeOverride, TimeStepMode, TimeWrapMode, advance_continuous,
    apply_time_override,
};

#[test]
fn continuous_time_advances_and_wraps() {
    let time = TimeOfDay::with_days(23.5, 2);
    let step = advance_continuous(time, 60.0, 60.0, 1.0, false, TimeWrapMode::Loop);

    assert_eq!(step.mode, TimeStepMode::Continuous);
    assert!((step.current.hour - 0.5).abs() < 1e-4);
    assert_eq!(step.current.elapsed_days, 3);
}

#[test]
fn paused_time_is_idle() {
    let time = TimeOfDay::new(6.0);
    let step = advance_continuous(time, 1.0, 60.0, 1.0, true, TimeWrapMode::Loop);

    assert_eq!(step.mode, TimeStepMode::Idle);
    assert_eq!(step.current, time);
}

#[test]
fn clamp_mode_stops_at_twenty_four() {
    let time = TimeOfDay::new(23.5);
    let step = advance_continuous(time, 120.0, 60.0, 1.0, false, TimeWrapMode::Clamp);

    assert_eq!(step.current.hour, DAY_LENGTH_HOURS);
    assert_eq!(step.current.elapsed_days, 0);
}

#[test]
fn scrub_sets_exact_hour_without_advancing_days() {
    let time = TimeOfDay::with_days(23.0, 7);
    let step = apply_time_override(time, TimeOverride::Scrub { hour: 5.25 }, TimeWrapMode::Loop);

    assert_eq!(step.mode, TimeStepMode::Scrub);
    assert!((step.current.hour - 5.25).abs() < 1e-4);
    assert_eq!(step.current.elapsed_days, 7);
}

#[test]
fn advance_override_crosses_midnight_and_counts_days() {
    let time = TimeOfDay::with_days(22.0, 1);
    let step = apply_time_override(
        time,
        TimeOverride::AdvanceHours { hours: 5.5 },
        TimeWrapMode::Loop,
    );

    assert_eq!(step.mode, TimeStepMode::AdvanceJump);
    assert!((step.current.hour - 3.5).abs() < 1e-4);
    assert_eq!(step.current.elapsed_days, 2);
}
