//! Tests: dimension tracking, derived units, conversions, error messages.

use unitx::parser::{convert, evaluate, ParseError};
use unitx::units::Dimension;

#[test]
fn plain_numbers_and_units() {
    let q = evaluate("42").unwrap();
    assert_eq!(q.si, 42.0);
    assert_eq!(q.dim, Dimension::DIMENSIONLESS);

    let q = evaluate("5 km").unwrap();
    assert_eq!(q.si, 5000.0);
    assert_eq!(q.dim, Dimension::base("m").unwrap());
}

#[test]
fn compound_units_track_dimensions() {
    let q = evaluate("90 km/h").unwrap();
    assert_eq!(q.dim, Dimension::base("m").unwrap() / Dimension::base("s").unwrap());
    // 90 km/h = 25 m/s
    assert!((q.si - 25.0).abs() < 1e-9);
}

#[test]
fn multiplication_cancels_dimensions() {
    // 3.5 m/s * 2 s = 7 m
    let q = evaluate("3.5 m/s * 2 s").unwrap();
    assert!((q.si - 7.0).abs() < 1e-9);
    assert_eq!(q.dim, Dimension::base("m").unwrap());
}

#[test]
fn adding_mismatched_dimensions_is_rejected() {
    let err = evaluate("1 m + 1 s").unwrap_err();
    match err {
        ParseError::Dimension(message) => {
            assert!(message.contains("cannot add"), "{}", message);
            assert!(message.contains("m") && message.contains("s"), "{}", message);
        }
        other => panic!("expected dimension error, got {:?}", other),
    }
}

#[test]
fn adding_matching_dimensions_works() {
    // 1 N + 1 kg*m/s^2: both force
    let q = evaluate("1 N + 1 kg*m/s^2").unwrap();
    assert!((q.si - 2.0).abs() < 1e-9);
    assert_eq!(q.dim.describe(), "N (kg·m/s²)");
}

#[test]
fn powers_apply_to_units_and_quantities() {
    let q = evaluate("(2 m)^3").unwrap();
    assert!((q.si - 8.0).abs() < 1e-9);
    assert_eq!(q.dim, Dimension::base("m").unwrap().pow(3));

    let q = evaluate("9.81 m/s^2").unwrap();
    assert_eq!(q.dim, Dimension::base("m").unwrap() / Dimension::base("s").unwrap().pow(2));
}

#[test]
fn derived_units_resolve_through_the_parser() {
    // 1 W = 1 kg*m^2/s^3
    let q = evaluate("5 W").unwrap();
    assert_eq!(q.dim, Dimension::base("kg").unwrap() * Dimension::base("m").unwrap().pow(2) / Dimension::base("s").unwrap().pow(3));

    let q = evaluate("1 bar / 1 atm").unwrap();
    assert_eq!(q.dim, Dimension::DIMENSIONLESS);
    assert!((q.si - 101325.0f64.recip() * 100000.0).abs() < 1e-9);
}

#[test]
fn convert_matches_dimensions() {
    let (q, converted) = convert(5.0, "km", "m").unwrap();
    assert!((converted - 5000.0).abs() < 1e-9);
    assert!((q.si - 5000.0).abs() < 1e-9);

    let (q, converted) = convert(1.0, "lb", "kg").unwrap();
    assert!((converted - 0.45359237).abs() < 1e-9);
    assert_eq!(q.dim, Dimension::base("kg").unwrap());
}

#[test]
fn convert_rejects_cross_dimension_requests() {
    let err = convert(5.0, "km", "s").unwrap_err();
    match err {
        ParseError::Dimension(message) => {
            assert!(message.contains("cannot convert"), "{}", message);
        }
        other => panic!("expected dimension error, got {:?}", other),
    }
}

#[test]
fn unknown_units_name_the_offender() {
    let err = evaluate("3Parse parse").err().unwrap();
    match err {
        ParseError::UnknownUnit { name } => assert_eq!(name, "Parse"),
        other => panic!("expected unknown unit, got {:?}", other),
    }
}

#[test]
fn syntax_errors_carry_offsets() {
    let err = evaluate("2 +").unwrap_err();
    match err {
        ParseError::Syntax { at, .. } => assert!(at >= 2),
        other => panic!("expected syntax error, got {:?}", other),
    }
}

#[test]
fn division_by_zero_is_reported() {
    let err = evaluate("5 m / 0 s").unwrap_err();
    match err {
        ParseError::Dimension(message) => assert!(message.contains("zero"), "{}", message),
        other => panic!("expected division error, got {:?}", other),
    }
}

#[test]
fn parentheses_and_negation() {
    let q = evaluate("-(2 km + 500 m)").unwrap();
    assert!((q.si + 2500.0).abs() < 1e-9);

    let q = evaluate("(1 min + 59 s) / 1 s").unwrap();
    assert!((q.si - 119.0).abs() < 1e-9);
    assert_eq!(q.dim, Dimension::DIMENSIONLESS);
}
