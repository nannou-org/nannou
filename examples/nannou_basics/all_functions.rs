use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {}

fn model(app: &App) -> Model {
    app.new_window()
        .size(720, 720)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .received_character(received_character)
        .mouse_moved(mouse_moved)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .mouse_wheel(mouse_wheel)
        .mouse_entered(mouse_entered)
        .mouse_exited(mouse_exited)
        .touch(touch)
        .moved(window_moved)
        .resized(window_resized)
        .hovered_file(hovered_file)
        .hovered_file_cancelled(hovered_file_cancelled)
        .dropped_file(dropped_file)
        .focused(window_focused)
        .unfocused(window_unfocused)
        .closed(window_closed)
        .build();
    Model {}
}

fn update(_app: &App, _model: &mut Model) {}

fn view(app: &App, _model: &Model, _entity: Entity) {
    let draw = app.draw();
    draw.background().color(SKY_BLUE);
}

fn key_pressed(_app: &App, _model: &mut Model, key: KeyCode) {
	println!("key pressed: {:?}", key);
}

fn key_released(_app: &App, _model: &mut Model, key: KeyCode) {
	println!("key released: {:?}", key);
}

fn received_character(_app: &App, _model: &mut Model, c: char) {
	println!("received character: {}", c);
}

fn mouse_moved(_app: &App, _model: &mut Model, pos: Point2) {
	println!("mouse moved: {:?}", pos);
}

fn mouse_pressed(_app: &App, _model: &mut Model, button: MouseButton) {
	println!("mouse pressed: {:?}", button);
}

fn mouse_released(_app: &App, _model: &mut Model, button: MouseButton) {
	println!("mouse released: {:?}", button);
}

fn mouse_wheel(_app: &App, _model: &mut Model, wheel: MouseWheel) {
	println!("mouse wheel: {:?}", wheel);
}

fn mouse_entered(_app: &App, _model: &mut Model) {
	println!("mouse entered");
}

fn mouse_exited(_app: &App, _model: &mut Model) {
	println!("mouse exited");
}

fn touch(_app: &App, _model: &mut Model, touch: TouchInput) {
	println!("touch: {:?}", touch);
}

fn window_moved(_app: &App, _model: &mut Model, pos: IVec2) {
	println!("window moved: {:?}", pos);
}

fn window_resized(_app: &App, _model: &mut Model, dim: Vec2) {
	println!("window resized: {:?}", dim);
}

fn window_focused(_app: &App, _model: &mut Model) {
	println!("window focused");
}

fn window_unfocused(_app: &App, _model: &mut Model) {
	println!("window unfocused");
}

fn window_closed(_app: &App, _model: &mut Model) {
	println!("window closed");
}

fn hovered_file(_app: &App, _model: &mut Model, path: std::path::PathBuf) {
	println!("hovered file: {:?}", path);
}

fn hovered_file_cancelled(_app: &App, _model: &mut Model) {
	println!("hovered file cancelled");
}

fn dropped_file(_app: &App, _model: &mut Model, path: std::path::PathBuf) {
	println!("dropped file: {:?}", path);
}
