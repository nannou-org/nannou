extern crate nannou;

use nannou::prelude::*;

#[test]
fn angle_test() {
    let vector = Vector2::new(1.0, 1.0);
    assert_eq!(vector.angle(), 0.7853981633974483);
    let vector = Vector2::new(5.0, 2.0);
    assert_eq!(vector.angle(), 0.3805063771123649);
    let vector = Vector2::new(-3.0, 4.0);
    assert_eq!(vector.angle(), 2.214297435588181);
    let vector = Vector2::new(-2.1, -6.7);
    assert_eq!(vector.angle(), -1.8745308126374836);
    let vector = Vector2::new(70.7, -60.8);
    assert_eq!(vector.angle(), -0.7102547457375739);
}

#[test]
fn limit_magnitude_test() {
    let vector = Vector2::new(10.0, 10.0);
    let vector = vector.limit_magnitude(2.0.sqrt());
    assert_eq!(vector, vec2(1.0, 1.0));
}
