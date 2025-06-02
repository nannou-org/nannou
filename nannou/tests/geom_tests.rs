use nannou::prelude::*;

#[test]
fn angle_test() {
    let vector = vec2(1.0, 1.0);
    assert_eq!(vector.angle(), 0.785_398_2);
    let vector = vec2(5.0, 2.0);
    assert_eq!(vector.angle(), 0.380_506_37);
    let vector = vec2(-3.0, 4.0);
    assert_eq!(vector.angle(), 2.214_297_5);
    let vector = vec2(-2.1, -6.7);
    assert_eq!(vector.angle(), -1.874531);
    let vector = vec2(70.7, -60.8);
    assert_eq!(vector.angle(), -0.710_254_7);
}
