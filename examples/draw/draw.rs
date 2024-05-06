use nannou::prelude::draw::instanced::InstanceData;
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(CORNFLOWER_BLUE);

    let instances = (-100..100)
        .into_iter()
        .map(|x| x as f32 / 10.0)
        .flat_map(|x| (-100..100).into_iter().map(move |y| (x, y as f32 / 10.0)))
        .collect::<Vec<(f32, f32)>>();

    draw.instanced()
        .with(draw.ellipse(), instances, |(i, j)| {
            InstanceData {
                position: Vec3::new(
                    *i * 100.0 / 5.0,
                    *j * 100.0 / 5.0,
                    *i / 10.0,
                ),
                scale: [i / 10.0;4],
                color: LinearRgba::from(Color::hsla(i / 100.0 * 360.0, *j, 0.5, 1.0)).to_f32_array(),
            }
        });
}
