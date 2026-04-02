use super::{DayPhase, DayPhaseBoundaries};

#[test]
fn every_time_belongs_to_exactly_one_phase() {
    let boundaries = DayPhaseBoundaries::default();
    for hour in 0..240 {
        let phase = boundaries.phase_at(hour as f32 * 0.1);
        assert!(matches!(
            phase,
            DayPhase::Dawn | DayPhase::Day | DayPhase::Dusk | DayPhase::Night
        ));
    }
}

#[test]
fn night_range_crosses_midnight() {
    let boundaries = DayPhaseBoundaries::default();
    assert_eq!(boundaries.phase_at(23.0), DayPhase::Night);
    assert_eq!(boundaries.phase_at(2.0), DayPhase::Night);
    assert_eq!(boundaries.phase_at(6.0), DayPhase::Dawn);
}

#[test]
fn phases_between_returns_boundary_order_once() {
    let boundaries = DayPhaseBoundaries::default();
    let phases = boundaries.phases_started_between(17.5, 30.0);
    assert_eq!(
        phases,
        vec![DayPhase::Dusk, DayPhase::Night, DayPhase::Dawn]
    );
}

#[test]
fn next_start_wraps_to_next_day() {
    let boundaries = DayPhaseBoundaries::default();
    let next_day = boundaries.next_start(DayPhase::Day, 18.0);
    assert!((next_day - 31.0).abs() < 1e-4);
}
