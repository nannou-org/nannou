use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    app.new_window().size(512, 512).view(view).build();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = app.asset_server().load(img_path);

    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);
    let win_rect = app.window_rect();

    // Generate the triangulated points for a cuboid to use for out mesh.
    let centre = pt3(0.0, 0.0, 0.0);
    let size = vec3(1.0, 1.0, 1.0);
    let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
    let points = cuboid
        .triangles_iter()
        .flat_map(geom::Tri::vertices)
        .map(|point| {
            // Tex coords should be in range (0.0, 0.0) to (1.0, 1.0);
            // This will have the logo show on the front and back faces.
            let [x, y, _] = point;
            let tex_coords = [x + 0.5, 1.0 - (y + 0.5)];
            (point, tex_coords)
        });

    // Scale the points up to half the window size.
    let cube_side = win_rect.w().min(win_rect.h()) * 0.5;
    let t = app.time();
    draw.scale(cube_side)
        .mesh()
        .points_textured(model.texture.clone(), points)
        .z_radians(t * 0.33)
        .x_radians(t * 0.166 + -app.mouse().x / 100.0)
        .y_radians(t * 0.25 + app.mouse().x / 100.0);

    // Draw to the frame!
}
