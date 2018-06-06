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
extern crate nannou;
use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    color_count: i32,
    _act_random_seed: i32,
    hue_values: Vec<f32>,
    saturation_values: Vec<f32>,
    brightness_values: Vec<f32>,
}

fn model(app: &App) -> Model {
    let color_count = 20;
    let _act_random_seed = 0;

    // Note you can decalre and pack a vector with random values like this in rust
    let hue_values = (0..color_count).map(|_| 0.0).collect();
    let saturation_values = (0..color_count).map(|_| 0.0).collect();
    let brightness_values = (0..color_count).map(|_| 0.0).collect();

    let _window = app.new_window().with_dimensions(720, 720).build().unwrap();
    Model {
        color_count,
        _act_random_seed,
        hue_values,
        saturation_values,
        brightness_values,
    }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        // ------ colors ------
        // create palette
        for i in 0..model.color_count {
            if i % 2 == 0 {
                model.hue_values[i as usize] = random::<f32>(); // * 0.36 + 0.61;
                model.saturation_values[i as usize] = 1.0;
                model.brightness_values[i as usize] = random::<f32>() * 0.85 + 0.15;
            } else {
                model.hue_values[i as usize] = 0.54;
                model.saturation_values[i as usize] = random::<f32>() * 0.8 + 0.2;
                model.brightness_values[i as usize] = 1.0;
            }
        }
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();

    // ------ area tiling ------
    // count tiles
    let mut counter = 0;
    // row count and row height
    let row_count = (random::<f32>() * 25.0 + 5.0) as i32;
    let row_height = (app.window_rect().h() as i32 / row_count) as i32;

    // seperate each line in parts
    for i in (0..row_count).rev() {
        // how many fragments
        let mut part_count = i + 1;
        let mut parts = Vec::new();

        let mut ii = 0;
        while ii < part_count {
            // sub fragment of not?
            if random::<f32>() < 0.075 {
                // take care of big values
                let fragments = (random::<f32>() * 18.0 + 2.0) as i32;
                part_count = part_count + fragments;
                for _ in 0..fragments {
                    parts.push(random::<f32>() * 2.0);
                }
            } else {
                parts.push(random::<f32>() * 18.0 + 2.0);
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

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
