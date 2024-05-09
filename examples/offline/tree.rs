// Daily Sketch 2019-04-22
// Alexis Andre (@mactuitui)

mod colors;
mod quadtree;
use crate::colors::Palette;
use nannou::prelude::*;

const LENGTH_FRAME: u64 = 700;
//const START_FRAME: u64 = 0;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    palette: Palette,
    things: Vec<Thing>,
}

// a Thing will be our main object, it'll try to grow outward
struct Thing {
    position: Vec2,
    size: f32,
    energy: f32,
    frac: f32,
    alive: bool,
    grown: bool,
    generation: usize,
    parent: Option<usize>,
    children: Vec<usize>,
}
// we only care if they are on the same spot
impl PartialEq for Thing {
    fn eq(&self, other: &Thing) -> bool {
        self.position == other.position
    }
}
// define what to use for the quadtree
impl quadtree::WithPos for Thing {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
}

impl Thing {
    fn new(x: f32, y: f32, s: f32, f: f32, parent: Option<usize>) -> Self {
        let position = pt2(x, y);
        let size = s;
        let frac = f;

        Thing {
            position,
            size,
            energy: 0.0,
            frac,
            alive: true,
            grown: false,
            generation: 0,
            parent,
            children: Vec::new(),
        }
    }

    fn distancept(&self, x: f32, y: f32) -> f32 {
        self.position.distance(vec2(x, y))
    }
    fn distance(&self, other: &Thing) -> f32 {
        self.position.distance(other.position)
    }
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1024, 1024)
        .view(view)
        .build();

    //create the random values we will need each frame

    let palette = Palette::new();

    let mut things = Vec::new();

    //we'll start from one thing in the middle
    let x = 0.0;
    let y = 0.0;
    let size = 1.0;
    let frac = 0.0;
    let candidate = Thing::new(x, y, size, frac, None);
    things.push(candidate);

    Model {
        palette: palette,
        things,
    }
}

fn update(app: &App, model: &mut Model) {
    //try to grow each circle until it hits another one

    //recreate the tree
    let mut tree = quadtree::QuadTree::new();
    for i in 0..model.things.len() {
        model.things[i].grown = false;
    }
    //grow and branch out?
    let max_count = model.things.len();
    for i in 0..model.things.len() {
        tree.insert(&model.things, i);
    }
    for i in 0..max_count {
        if model.things[i].parent == None {
            if app.elapsed_frames() < 1000 {
                model.things[i].energy += 10000.0;
                if model.things[i].alive == true {
                    model.things[i].grown = true;
                }
            }
        }
        //move the size to the children
        if model.things[i].energy > 10.0 {
            //push the size to children
            //or branch
            let prob = 0.2;

            //if no child create a one

            //create a child
            if random_f32() < prob {
                let mut angle = random_f32() * TAU;
                if let Some(parent) = model.things[i].parent {
                    let from_parent = model.things[i].position - model.things[parent].position;
                    let base_angle = from_parent.y.atan2(from_parent.x);
                    let mut diff = (random_f32() - 0.5) * 2.0;
                    diff = diff.powf(19.0);
                    angle = base_angle + diff * PI;
                }

                let r = model.things[i].size + 1.0;
                let x = model.things[i].position.x + r * angle.cos();
                let y = model.things[i].position.y + r * angle.sin();
                let s = 1.0;
                let mut candidate = Thing::new(x, y, s, angle / PI, Some(i));
                candidate.generation = model.things[i].generation + 1;
                let indices = tree.get_elements(&model.things, x, y, 50.0);
                let mut ok = true;
                for k in 0..indices.len() {
                    let d = model.things[indices[k]].distancept(x, y);
                    if d < model.things[indices[k]].size + 1.0 {
                        ok = false;
                    }
                }
                if x.abs() > 450.0 || y.abs() > 450.0 {
                    ok = false;
                }
                if ok {
                    model.things[i].alive = false;
                    model.things[i].energy -= 1.0;
                    model.things.push(candidate);
                    let s = model.things.len() - 1;
                    model.things[i].children.push(s);
                }
            }
            if model.things[i].energy > 10.0 {
                if model.things[i].children.len() > 0 {
                    model.things[i].energy -= 1.0;
                    for k in 0..model.things[i].children.len() {
                        let other = model.things[i].children[k];
                        model.things[other].energy += 3.0;
                        model.things[other].grown = true;
                    }
                }
            }
        }
    }

    //check if the grown things are free
    for i in 0..model.things.len() {
        if model.things[i].grown == true {
            let indices = tree.get_elements(
                &model.things,
                model.things[i].position.x,
                model.things[i].position.y,
                60.0,
            );
            for k in 0..indices.len() {
                let mut ok = true;
                if let Some(parent) = model.things[i].parent {
                    if parent == indices[k] {
                        ok = false;
                    }
                }
                if i == indices[k] {
                    ok = false;
                }
                if ok {
                    //we can check this one
                    let d = model.things[i].distance(&model.things[indices[k]]);
                    if d < model.things[i].size + model.things[indices[k]].size + 1.0 {
                        model.things[i].alive = false;
                    }
                }
            }
            if model.things[i].alive == true {
                if model.things[i].size > 29.0 {
                    model.things[i].alive = false;
                } else {
                    //grow it
                    if let Some(parent) = model.things[i].parent {
                        model.things[i].size += 1.0;
                        let direction =
                            (model.things[i].position - model.things[parent].position).normalize();
                        model.things[i].position = model.things[parent].position
                            + direction * (model.things[i].size + model.things[parent].size);
                    } else {
                        model.things[i].size += 1.0;
                    }
                }
            }
        }
    }
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background
    draw.background().color(WHITE);

    let scale = 1.0;

    //how far are we from the end -> to fade out
    let mut frac_end = (((app.elapsed_frames() + 120) as i32 - LENGTH_FRAME as i32) as f32) / 100.0;
    frac_end = frac_end.max(0.0).min(1.0);

    let c: Srgba = Color::srgba(1.0, 1.0, 1.0, 1.0 - frac_end).into();

    //draw ALL THE THINGS
    for k in 0..model.things.len() {
        //get a color from the palette indexed by frac
        let mut c2: Srgba  = model.palette.somecolor_frac(model.things[k].frac).into();
        // make it fade
        c2.alpha = 1.0 - frac_end;
        let c3 = Color::srgba(0.0, 0.0, 0.0, 1.0 - frac_end);

        //draw in three steps
        draw.ellipse()
            .resolution(20.0)
            .xy(model.things[k].position * scale)
            .w_h(
                model.things[k].size * 1.3 * scale,
                model.things[k].size * 1.3 * scale,
            )
            .color(c);
        draw.ellipse()
            .resolution(20.0)
            .xy(model.things[k].position * scale)
            .w_h(
                model.things[k].size * 1.2 * scale,
                model.things[k].size * 1.2 * scale,
            )
            .color(c2);
        draw.ellipse()
            .resolution(20.0)
            .xy(model.things[k].position * scale)
            .w_h(
                model.things[k].size * 0.5 * scale,
                model.things[k].size * 0.5 * scale,
            )
            .color(c3);

        //link to the children
        for l in 0..model.things[k].children.len() {
            draw.line()
                .start(model.things[k].position * scale)
                .end(model.things[model.things[k].children[l]].position * scale)
                .color(c3)
                .weight((model.things[model.things[k].children[l]].size * 0.5).min(5.0));
        }
    }

    // Write to the window frame.


    //TODO add screenshot
}
