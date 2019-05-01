/**
* generates a specific color palette and some random "rect-tilings"
*
* MOUSE
* left click          : new composition
*
* KEYS
* s                   : save png
* c                   : save color palette
*/
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    color_count: usize,
    hue_values: Vec<f32>,
    saturation_values: Vec<f32>,
    brightness_values: Vec<f32>,
}

fn model(_app: &App) -> Model {
    let color_count = 20;

    // Note you can decalre and pack a vector with random values like this in rust
    let hue_values = vec![0.0; color_count];
    let saturation_values = vec![0.0; color_count];
    let brightness_values = vec![0.0; color_count];

    Model {
        color_count,
        hue_values,
        saturation_values,
        brightness_values,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Create palette
    for i in 0..model.color_count {
        if i % 2 == 0 {
            model.hue_values[i] = random_f32(); // * 0.36 + 0.61;
            model.saturation_values[i] = 1.0;
            model.brightness_values[i] = random_f32() * 0.85 + 0.15;
        } else {
            model.hue_values[i] = 0.54;
            model.saturation_values[i] = random_f32() * 0.8 + 0.2;
            model.brightness_values[i] = 1.0;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();

    // ------ area tiling ------
    // count tiles
    let mut counter = 0;
    // row count and row height
    let row_count = (random_f32() * 25.0 + 5.0) as i32;
    let row_height = (app.window_rect().h() as i32 / row_count) as i32;

    // seperate each line in parts
    for i in (0..row_count).rev() {
        // how many fragments
        let mut part_count = i + 1;
        let mut parts = Vec::new();

        let mut ii = 0;
        while ii < part_count {
            // sub fragment of not?
            if random_f32() < 0.075 {
                // take care of big values
                let fragments = (random_f32() * 18.0 + 2.0) as i32;
                part_count = part_count + fragments;
                for _ in 0..fragments {
                    parts.push(random_f32() * 2.0);
                }
            } else {
                parts.push(random_f32() * 18.0 + 2.0);
            }
            ii += 1;
        }

        // add all subparts
        let mut sum_parts_total = 0.0;
        for ii in 0..part_count {
            sum_parts_total += parts[ii as usize];
        }

        // draw rects
        let mut sum_parts_now = 0.0;
        for ii in 0..parts.len() {
            sum_parts_now += parts[ii as usize];

            let x = map_range(
                sum_parts_now,
                0.0,
                sum_parts_total,
                app.window_rect().left(),
                app.window_rect().right(),
            );
            let y = app.window_rect().top() - (row_height * i) as f32;
            let w = -map_range(
                parts[ii],
                0.0,
                sum_parts_total,
                app.window_rect().left(),
                app.window_rect().right(),
            );
            let h = row_height as f32;

            let index = counter % model.color_count;
            draw.rect().x_y(x, y).w_h(w, h).hsv(
                model.hue_values[index as usize],
                model.saturation_values[index as usize],
                model.brightness_values[index as usize],
            );

            counter += 1;
        }
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
