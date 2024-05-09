// P_4_3_1_02
//
// Generative Gestaltung – Creative Coding im Web
// ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
// Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
// with contributions by Joey Lee and Niels Poldervaart
// Copyright 2018
//
// http://www.generative-gestaltung.de
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
/**
 * pixel mapping. each pixel is translated into a new element (svg file).
 * take care to sort the svg file according to their greyscale value.
 *
 * KEYS
 * s                   : save png
 */
use nannou::prelude::*;

use nannou::image;
use nannou::image::GenericImageView;
use nannou::lyon::math::Point;
use nannou::lyon::path::PathEvent;

fn main() {
    nannou::app(model).run();
}

struct Model {
    image: image::DynamicImage,
    shapes: Vec<SvgPath>,
}

#[derive(Clone)]
struct SvgPath {
    events: Vec<PathEvent>,
    weight: f32,
    color: Srgba,
}

impl SvgPath {
    fn new(events: Vec<PathEvent>, weight: f32, color: Srgba) -> Self {
        SvgPath {
            events,
            weight,
            color,
        }
    }
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window()
        .size(600, 900)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let svg_assets_path = app
        .assets_path()
        .unwrap()
        .join("svg")
        .join("generative_examples");

    let mut assets = Vec::new();
    assets.push(svg_assets_path.join("056.svg"));
    assets.push(svg_assets_path.join("076.svg"));
    assets.push(svg_assets_path.join("082.svg"));
    assets.push(svg_assets_path.join("096.svg"));
    assets.push(svg_assets_path.join("117.svg"));
    assets.push(svg_assets_path.join("148.svg"));
    assets.push(svg_assets_path.join("152.svg"));
    assets.push(svg_assets_path.join("157.svg"));
    assets.push(svg_assets_path.join("164.svg"));
    assets.push(svg_assets_path.join("166.svg"));
    assets.push(svg_assets_path.join("186.svg"));
    assets.push(svg_assets_path.join("198.svg"));
    assets.push(svg_assets_path.join("224.svg"));

    let mut shapes = Vec::new();

    for asset in assets {
        let opt = usvg::Options::default();
        let rtree = usvg::Tree::from_file(&asset, &opt).unwrap();

        for node in rtree.root().descendants() {
            if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                if let Some(ref stroke) = p.stroke {
                    let color = match stroke.paint {
                        usvg::Paint::Color(c) => Color::srgba(
                            c.red as f32 / 255.0,
                            c.green as f32 / 255.0,
                            c.blue as f32 / 255.0,
                            1.0,
                        ),
                        _ => Color::srgba(0.0, 0.0, 0.0, 1.0),
                    };

                    let path_events = convert_path(p);
                    let mut v = Vec::new();
                    for e in path_events {
                        v.push(e);
                    }
                    let path = SvgPath::new(v, stroke.width.value() as f32, color);
                    shapes.push(path);
                }
            }
        }
    }

    let img_path = app
        .assets_path()
        .unwrap()
        .join("images")
        .join("generative_examples")
        .join("p_4_3_1_01.png");

    let image = image::open(img_path).unwrap();

    Model { image, shapes }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    draw.background().color(WHITE);

    let draw = app.draw();
    let win = app.window_rect();

    let (w, h) = model.image.dimensions();
    for grid_x in 0..w {
        for grid_y in 0..h {
            // get current color
            let c = model.image.get_pixel(grid_x, grid_y);
            // greyscale conversion
            let red = c[0] as f32 / 255.0;
            let green = c[1] as f32 / 255.0;
            let blue = c[2] as f32 / 255.0;
            let greyscale = red * 0.222 + green * 0.707 + blue * 0.071;
            let gradient_to_index = map_range(greyscale, 0.0, 1.0, 0, model.shapes.len() - 1);

            // Grid position + tile size
            let tile_width = 603.0 / w as f32;
            let tile_height = 873.0 / h as f32;
            let pos_x = win.left() + tile_width * grid_x as f32 + (tile_width / 2.0);
            let pos_y = win.top() - tile_height * grid_y as f32 - (tile_height / 2.0);

            let shape = &model.shapes[gradient_to_index];
            let weight = shape.weight;
            let _c = shape.color;
            let e = shape.events.iter().cloned();

            draw.path()
                .stroke()
                .stroke_weight(weight)
                .rgb(red, green, blue)
                .events(e)
                .x_y(pos_x, pos_y);
        }
    }

}

fn key_released(app: &App, _model: &mut Model, key: Key) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}

/// Some glue between usvg's iterators and lyon's.

fn point(x: &f64, y: &f64) -> Point {
    Point::new((*x) as f32, (*y) as f32)
}

pub struct PathConvIter<'a> {
    iter: std::slice::Iter<'a, usvg::PathSegment>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<PathEvent>,
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = PathEvent;
    fn next(&mut self) -> Option<PathEvent> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }

        let next = self.iter.next();
        match next {
            Some(usvg::PathSegment::MoveTo { x, y }) => {
                if self.needs_end {
                    let last = self.prev;
                    let first = self.first;
                    self.needs_end = false;
                    self.prev = point(x, y);
                    self.deferred = Some(PathEvent::Begin { at: self.prev });
                    self.first = self.prev;
                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    self.first = point(x, y);
                    Some(PathEvent::Begin { at: self.first })
                }
            }
            Some(usvg::PathSegment::LineTo { x, y }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = point(x, y);
                Some(PathEvent::Line {
                    from,
                    to: self.prev,
                })
            }
            Some(usvg::PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = point(x, y);
                Some(PathEvent::Cubic {
                    from,
                    ctrl1: point(x1, y1),
                    ctrl2: point(x2, y2),
                    to: self.prev,
                })
            }
            Some(usvg::PathSegment::ClosePath) => {
                self.needs_end = false;
                self.prev = self.first;
                Some(PathEvent::End {
                    last: self.prev,
                    first: self.first,
                    close: true,
                })
            }
            None => {
                if self.needs_end {
                    self.needs_end = false;
                    let last = self.prev;
                    let first = self.first;
                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub fn convert_path<'a>(p: &'a usvg::Path) -> PathConvIter<'a> {
    PathConvIter {
        iter: p.segments.iter(),
        first: Point::new(0.0, 0.0),
        prev: Point::new(0.0, 0.0),
        deferred: None,
        needs_end: false,
    }
}
