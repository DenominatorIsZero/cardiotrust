use approx::assert_relative_eq;

use crate::core::model::functional::allpass::{
    from_samples_to_coef, from_samples_to_usize, offset_to_gain_index,
};

#[test]
fn from_samples_to_usize_1() {
    assert_eq!(1, from_samples_to_usize(1.0));
    assert_eq!(1, from_samples_to_usize(1.2));
    assert_eq!(10, from_samples_to_usize(10.9));
    assert_eq!(10, from_samples_to_usize(10.0));
}

#[test]
fn from_samples_to_coef_1() {
    assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(0.5));
    assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(1.5));
    assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(99999.5));

    assert_relative_eq!(0.9999, from_samples_to_coef(0.0));
    assert_relative_eq!(0.9999, from_samples_to_coef(1.0));
    assert_relative_eq!(0.9999, from_samples_to_coef(99999.0));
}

#[test]
fn offset_to_index_test() {
    let desired = 2;
    let actual = offset_to_gain_index(-1, -1, -1, 2).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 5;
    let actual = offset_to_gain_index(-1, -1, 0, 2).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 8;
    let actual = offset_to_gain_index(-1, -1, 1, 2).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 77;
    let actual = offset_to_gain_index(1, 1, 1, 2).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 42;
    let actual = offset_to_gain_index(0, 1, -1, 0).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 45;
    let actual = offset_to_gain_index(0, 1, 0, 0).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 60;
    let actual = offset_to_gain_index(1, 0, -1, 0).expect("Offsets to be valid.");
    assert_eq!(desired, actual);

    let desired = 63;
    let actual = offset_to_gain_index(1, 0, 0, 0).expect("Offsets to be valid.");
    assert_eq!(desired, actual);
}
