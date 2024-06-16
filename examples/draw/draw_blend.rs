use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Change the background luminance based on mouse x.
    let w = app.window_rect();
    let lum = map_range(app.mouse().x, w.left(), w.right(), 0.0, 1.0);
    let clear = Color::gray(lum);
    // draw.background().color(clear);

    // Put all the provided blend modes in a list.
    let blends = [
        ("NORMAL", BLEND_NORMAL),
        ("ADD", BLEND_ADD),
        ("SUBTRACT", BLEND_SUBTRACT),
        ("REVERSE SUBTRACT", BLEND_REVERSE_SUBTRACT),
        ("DARKEST", BLEND_DARKEST),
        ("LIGHTEST", BLEND_LIGHTEST),
    ];

    // Select a color blend descriptor based on mouse y.
    let ix = map_range(app.mouse().x, w.top(), w.bottom(), 0, blends.len());
    let ix = std::cmp::min(ix, blends.len() - 1);
    let (blend_name, desc) = &blends[ix];

    // Draw the name of the blend mode and its descriptor.
    let mut draw = app.draw();
    draw.background().color(clear);
    let color = Color::gray(1.0f32 - lum.round());
    // draw.text(blend_name)
    //     .color(color)
    //     .font_size(48)
    //     .wh(w.wh() * 0.7)
    //     .align_text_top();
    // let text = format!("{:?}", desc);
    // draw.text(&text)
    //     .color(color)
    //     .wh(w.wh() * 0.8)
    //     .align_text_bottom();

    // Assign the blend mode.
    // let mut draw = draw.color_blend(desc.clone());

    // Draw RGB circles.
    let t = app.elapsed_seconds();
    let n_circles = 3;
    let radius = w.right().min(w.top()) * 0.5 / n_circles as f32;
    let animate_radius = -((t.sin() * 0.5 + 0.5) * radius * 0.5);
    draw = draw.x(w.left() * 0.5).color_blend(*desc);
    for i in 0..n_circles {
        let hue = i as f32 / n_circles as f32;
        let color = Color::hsl(hue, 1.0, 0.5);
        draw.ellipse()
            // .color_blend(desc.clone())
            .radius(radius)
            .color(color)
            .x(radius + animate_radius);
        draw = draw.rotate(PI * 2.0 / n_circles as f32);
    }

    // Draw CMY.
    draw = draw.x(w.right() * 0.5).color_blend(*desc);
    for i in 0..n_circles {
        let hue = i as f32 / n_circles as f32;
        let color = Color::hsl(hue + 0.5, 1.0, 0.5);
        draw.ellipse()
            .radius(radius)
            .color(color)
            .x(radius + animate_radius);
        draw = draw.rotate(PI * 2.0 / n_circles as f32);
    }

    // Draw ascending luminance.
    draw = draw.x(w.right() * 0.5).color_blend(*desc);
    for i in 0..n_circles {
        let lum = (0.5 + i as f32) / n_circles as f32;
        let color = Color::gray(lum);
        draw.ellipse()
            .radius(radius)
            .color(color)
            .x(radius + animate_radius);
        draw = draw.rotate(PI * 2.0 / n_circles as f32);
    }
}
