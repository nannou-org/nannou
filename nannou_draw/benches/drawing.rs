//! A simple timing benchmark for building a frame's worth of draw commands.
//!
//! Run with `cargo bench -p nannou_draw`.

use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::prelude::*;
use nannou_draw::draw::Draw;
use nannou_draw::text::font::{NannouTextCxInner, SharedTextCx};
use parley::{FontContext, LayoutContext};

const ITERS: usize = 20;
const ELLIPSES: usize = 10_000;
const POLYLINES: usize = 1_000;
const POLYLINE_POINTS: usize = 100;

fn main() {
    let text_cx = SharedTextCx(Arc::new(Mutex::new(NannouTextCxInner {
        font: FontContext::default(),
        layout: LayoutContext::new(),
    })));
    let mut draw: Draw = Draw::new(Entity::PLACEHOLDER, text_cx);

    let mut times = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        draw.reset();
        let start = Instant::now();
        build_frame(&draw);
        let cmds = draw.drain_commands().count();
        times.push(start.elapsed());
        black_box(cmds);
    }

    times.sort();
    let median = times[times.len() / 2];
    let min = times[0];
    let max = times[times.len() - 1];
    println!(
        "build_frame ({ELLIPSES} ellipses x 6 props + {POLYLINES} polylines x {POLYLINE_POINTS} \
         points): median {median:?}, min {min:?}, max {max:?} over {ITERS} iters"
    );
}

fn build_frame(draw: &Draw) {
    for i in 0..ELLIPSES {
        let f = i as f32;
        draw.ellipse()
            .x_y(f % 100.0, f % 50.0)
            .radius(5.0 + (f % 10.0))
            .color(Color::srgb(0.5, 0.2, 0.8))
            .stroke(Color::WHITE)
            .stroke_weight(2.0)
            .rotate(f * 0.01);
    }
    for i in 0..POLYLINES {
        let f = i as f32;
        draw.polyline()
            .weight(2.0)
            .color(Color::srgb(0.1, 0.9, 0.4))
            .points((0..POLYLINE_POINTS).map(|p| Vec2::new(p as f32, (p as f32 + f).sin())));
    }
}
