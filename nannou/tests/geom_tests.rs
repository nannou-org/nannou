use std::matches;

use lyon::path::Event;
use nannou::geom::path;
use nannou::prelude::*;

#[test]
fn angle_test() {
    let vector = vec2(1.0, 1.0);
    assert_eq!(vector.angle(), 0.7853981633974483);
    let vector = vec2(5.0, 2.0);
    assert_eq!(vector.angle(), 0.3805063771123649);
    let vector = vec2(-3.0, 4.0);
    assert_eq!(vector.angle(), 2.214297435588181);
    let vector = vec2(-2.1, -6.7);
    assert_eq!(vector.angle(), -1.874531);
    let vector = vec2(70.7, -60.8);
    assert_eq!(vector.angle(), -0.7102547457375739);
}

#[test]
fn path_builder_end_test() {
    let mut builder = path();
    builder = builder.begin(vec2(-50.0, 0.0));
    builder = builder.quadratic_bezier_to(vec2(0.0, 25.0), vec2(50.0, 0.0));
    builder = builder.end();
    let path = builder.build();
    assert!(matches!(
        path.iter().last(),
        Some(Event::End { close: false, .. })
    ));
}

#[test]
fn path_builder_close_test() {
    let mut builder = path();
    builder = builder.begin(vec2(-50.0, 0.0));
    builder = builder.quadratic_bezier_to(vec2(0.0, 25.0), vec2(50.0, 0.0));
    builder = builder.close();
    let path = builder.build();
    assert!(matches!(
        path.iter().last(),
        Some(Event::End { close: true, .. })
    ));
}
