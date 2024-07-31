/*
 * Simple Loop example with 4D noise
 * Daily Sketch 2019/09/23 by Alexis Andre (@mactuitui)
 *
 * Demonstration of looping an animation using periodic functions.
 *
 */

use nannou::noise::*;
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    noise: Perlin,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size_pixels(1024, 1024).view(view).build();
    let mut noise = Perlin::new();
    noise = noise.set_seed(1);
    Model { noise }
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let draw = app.draw();
    draw.background().color(Color::BLACK);

    //the loop is going to be 200 frames long
    let frac = (app.elapsed_frames() % 200) as f32 / (200.0);

    //we'll rotate in the noise space
    let rotcos = 0.2 * (frac * TAU).cos();
    let rotsin = 0.2 * (frac * TAU).sin();

    //draw the lines
    for j in 0..190 {
        let frac_j = (j as f32) / 189.0;
        let mut pts = Vec::new();
        let mut pts2 = Vec::new();
        for i in 0..200 {
            let frac_i = (i as f32) / 199.0;
            let scale = ((frac_i * PI).sin()).powf(3.0);
            let offset = scale
                * (model.noise.get([
                    i as f64 * 0.03,
                    j as f64 * 0.5,
                    rotcos as f64,
                    rotsin as f64,
                ]) * 0.5
                    + 0.5) as f32;
            pts.push(vec2(
                -512.0 + frac_i * 1024.0,
                342.0 - frac_j * 824.0 + 160.0 * offset,
            ));
            pts2.push(vec2(
                -512.0 + frac_i * 1024.0,
                342.0 - frac_j * 824.0 + 160.0 * offset,
            ));
        }
        //fill the line with black
        draw.polygon().color(Color::BLACK).points(pts);
        //draw the white outline on top
        draw.polyline()
            .color(Color::WHITE)
            .stroke_weight(5.0)
            .points(pts2);
    }
}
