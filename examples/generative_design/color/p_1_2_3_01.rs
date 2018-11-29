/**
* generates specific color palettes
*
* MOUSE
* position x/y        : row and coloum count
*
* KEYS
* 0-9                 : creates specific color palettes
* s                   : save png
* c                   : save color palette
*/
extern crate nannou;
use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).view(view).run();
}

struct Model {
    tile_count_x: i32,
    tile_count_y: i32,
    hue_values: Vec<f32>,
    saturation_values: Vec<f32>,
    brightness_values: Vec<f32>,
}

fn model(app: &App) -> Model {
    let tile_count_x = 50;
    let tile_count_y = 10;

    let mut hue_values: Vec<f32> = Vec::new();
    let mut saturation_values: Vec<f32> = Vec::new();
    let mut brightness_values: Vec<f32> = Vec::new();

    for _i in 0..tile_count_x as i32 {
        hue_values.push(random());
        saturation_values.push(random());
        brightness_values.push(random());
    }

    let _window = app.new_window().with_dimensions(720, 720).build().unwrap();
    Model {
        tile_count_x,
        tile_count_y,
        hue_values,
        saturation_values,
        brightness_values,
    }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(key) => match key {
                    Key::Key1 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = random();
                            model.saturation_values[i as usize] = random();
                            model.brightness_values[i as usize] = random();
                        }
                    }
                    Key::Key2 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = random();
                            model.saturation_values[i as usize] = random();
                            model.brightness_values[i as usize] = 1.0;
                        }
                    }
                    Key::Key3 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = random();
                            model.saturation_values[i as usize] = 1.0;
                            model.brightness_values[i as usize] = random();
                        }
                    }
                    Key::Key4 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = 0.0;
                            model.saturation_values[i as usize] = 0.0;
                            model.brightness_values[i as usize] = random();
                        }
                    }
                    Key::Key5 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = 0.54;
                            model.saturation_values[i as usize] = 1.0;
                            model.brightness_values[i as usize] = random();
                        }
                    }
                    Key::Key6 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = 0.54;
                            model.saturation_values[i as usize] = random();
                            model.brightness_values[i as usize] = 1.0;
                        }
                    }
                    Key::Key7 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = random_f32() * 0.5;
                            model.saturation_values[i as usize] = random_f32() * 0.2 + 0.8;
                            model.brightness_values[i as usize] = random_f32() * 0.4 + 0.5;
                        }
                    }
                    Key::Key8 => {
                        for i in 0..model.tile_count_x as i32 {
                            model.hue_values[i as usize] = random_f32() * 0.5 + 0.5;
                            model.saturation_values[i as usize] = random_f32() * 0.2 + 0.8;
                            model.brightness_values[i as usize] = random_f32() * 0.4 + 0.5;
                        }
                    }
                    Key::Key9 => {
                        for i in 0..model.tile_count_x as i32 {
                            if i % 2 == 0 {
                                model.hue_values[i as usize] = random();
                                model.saturation_values[i as usize] = 1.0;
                                model.brightness_values[i as usize] = random();
                            } else {
                                model.hue_values[i as usize] = 0.54;
                                model.saturation_values[i as usize] = random();
                                model.brightness_values[i as usize] = 1.0;
                            }
                        }
                    }
                    Key::Key0 => {
                        for i in 0..model.tile_count_x as i32 {
                            if i % 2 == 0 {
                                model.hue_values[i as usize] = 0.38;
                                model.saturation_values[i as usize] = random_f32() * 0.7 + 0.3;
                                model.brightness_values[i as usize] = random_f32() * 0.6 + 0.4;
                            } else {
                                model.hue_values[i as usize] = 0.58;
                                model.saturation_values[i as usize] = random_f32() * 0.6 + 0.4;
                                model.brightness_values[i as usize] = random_f32() * 0.5 + 0.5;
                            }
                        }
                    }
                    _ => (),
                },
                _other => (),
            }
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();

    // white black
    draw.background().rgb(0.0, 0.0, 0.2);
    let win_rect = app.window_rect();

    // limit mouse coordintes to canvas
    let mx = (app.mouse.x - win_rect.left()).max(1.0).min(win_rect.w());
    let my = (win_rect.top() - app.mouse.y).max(1.0).min(win_rect.h());

    // tile counter
    let mut counter = 0;

    // map mouse to grid resolution
    let current_tile_count_x =
        map_range(mx, 0.0, win_rect.w(), 1.0, model.tile_count_x as f32) as i32;
    let current_tile_count_y =
        map_range(my, 0.0, win_rect.h(), 1.0, model.tile_count_y as f32) as i32;
    let tile_width = win_rect.w() as i32 / current_tile_count_x;
    let tile_height = win_rect.h() as i32 / current_tile_count_y;

    let size = vec2(tile_width as f32, tile_height as f32);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);
    let mut grid_y = 0;
    while grid_y < model.tile_count_y {
        let mut grid_x = 0;
        while grid_x < model.tile_count_x {
            let r = r
                .shift_x((tile_width * grid_x) as f32)
                .shift_y(-(tile_height * grid_y) as f32);
            let index = counter % current_tile_count_x as usize;
            draw.rect().xy(r.xy()).wh(r.wh()).hsv(
                model.hue_values[index],
                model.saturation_values[index],
                model.brightness_values[index],
            );
            counter += 1;
            grid_x += 1;
        }
        grid_y += 1;
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
