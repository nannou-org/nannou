use crate::Model;
use nannou::ui::prelude::*;
use nannou::prelude::*;
use nannou::geom::rect::Rect;
use self::ui::input::state::mouse::ButtonPosition;

pub const PAD_X: f32 = 20.0;
pub const PAD_Y: f32 = 20.0;

pub struct Controls {
    pub corners: Corners,
}

pub struct Corners {
    pub dims: Rect<f32>,
    pub top_left: Corner,
    pub top_right: Corner,
    pub bottom_left: Corner,
    pub bottom_right: Corner,
}

pub struct Corner {
    pub drag: bool,
    pub pos: Point2,

}

pub struct Ids {
    pub top_left_corner: widget::Id,
    pub top_right_corner: widget::Id,
    pub bottom_left_corner: widget::Id,
    pub bottom_right_corner: widget::Id,
    pub background: widget::Id,
    pub points: widget::Id,
    pub tl_text: widget::Id,
    pub tr_text: widget::Id,
    pub bl_text: widget::Id,
    pub br_text: widget::Id,
}

impl Corners {
    pub fn new(init: Rect<f32>) -> Self {
        Corners {
            dims: init,
            top_left: Corner{ drag: false, pos: pt2(init.x.start, init.y.end) },
            top_right: Corner{ drag: false, pos: pt2(init.x.end, init.y.end) },
            bottom_left: Corner{ drag: false, pos: pt2(init.x.start, init.y.start) },
            bottom_right: Corner{ drag: false, pos: pt2(init.x.end, init.y.start) },
        }
    }
}


pub(crate) fn update(model: &mut Model) {
    let ui = &mut model.ui.set_widgets();

    let ref mut corners = model.controls.corners;

    widget::Canvas::new().rgb(0.2, 0.0, 0.2).set(model.ids.background, ui);

    widget::Circle::fill(20.0)
        .rgb(0.0, 0.7, 0.0)
        .x(corners.top_left.pos.x as f64)
        .y(corners.top_left.pos.y as f64)
        .set(model.ids.top_left_corner, ui);

    widget::Text::new(&format!("top left: {:?}", corners.top_left.pos))
        .font_size(12)
        .rgb(1.0, 0.3, 0.0)
        .down(1.0)
        .set(model.ids.tl_text, ui);

    widget::Circle::fill(20.0)
        .rgb(0.0, 0.7, 0.0)
        .x_position(position::Position::Absolute(corners.top_right.pos.x as f64))
        .y_position(position::Position::Absolute(corners.top_right.pos.y as f64))
        .set(model.ids.top_right_corner, ui);

    widget::Text::new(&format!("top right: {:?}", corners.top_right.pos))
        .font_size(12)
        .rgb(1.0, 0.3, 0.0)
        .down(1.0)
        .set(model.ids.tr_text, ui);

    widget::Circle::fill(20.0)
        .rgb(0.0, 0.7, 0.0)
        .x_position(position::Position::Absolute(corners.bottom_left.pos.x as f64))
        .y_position(position::Position::Absolute(corners.bottom_left.pos.y as f64))
        .set(model.ids.bottom_left_corner, ui);

    widget::Text::new(&format!("bottom left: {:?}", corners.bottom_left.pos))
        .font_size(12)
        .rgb(1.0, 0.3, 0.0)
        .down(1.0)
        .set(model.ids.bl_text, ui);

    widget::Circle::fill(20.0)
        .rgb(0.0, 0.7, 0.0)
        .x_position(position::Position::Absolute(corners.bottom_right.pos.x as f64))
        .y_position(position::Position::Absolute(corners.bottom_right.pos.y as f64))
        .set(model.ids.bottom_right_corner, ui);

    widget::Text::new(&format!("bottom right: {:?}", corners.bottom_right.pos))
        .font_size(12)
        .rgb(1.0, 0.3, 0.0)
        .down(1.0)
        .set(model.ids.br_text, ui);

    let points = vec![
        corners.top_left.pos,
        corners.top_right.pos,
        corners.bottom_right.pos,
        corners.bottom_left.pos,
        corners.top_left.pos];
    widget::PointPath::new(points.into_iter().map(|v| [v.x as f64, v.y as f64]))
        .set(model.ids.points, ui);

    match (ui.global_input().current.widget_capturing_mouse, ui.global_input().current.mouse.buttons.left()) {
        (Some(id), ButtonPosition::Down(_, _)) if id == model.ids.top_left_corner => corners.top_left.drag = true,
        (Some(id), ButtonPosition::Down(_, _)) if id == model.ids.top_right_corner => corners.top_right.drag = true,
        (Some(id), ButtonPosition::Down(_, _)) if id == model.ids.bottom_left_corner => corners.bottom_left.drag = true,
        (Some(id), ButtonPosition::Down(_, _)) if id == model.ids.bottom_right_corner => corners.bottom_right.drag = true,
        _ => (),

    }
}

pub(crate) fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    let ref mut corners = model.controls.corners;
    match event {
        MouseMoved(pos) => {
            let pos = pt2(clamp(pos.x, corners.dims.x.start, corners.dims.x.end),
            clamp(pos.y, corners.dims.y.end, corners.dims.y.start));
            if corners.top_left.drag {
                corners.top_left.pos = pos;
            } else if corners.top_right.drag {
                corners.top_right.pos = pos;
            } else if corners.bottom_left.drag {
                corners.bottom_left.pos = pos;
            } else if corners.bottom_right.drag {
                corners.bottom_right.pos = pos;
            }
        },
        MouseReleased(b) if b == MouseButton::Left => {
            corners.top_left.drag = false;
            corners.top_right.drag = false;
            corners.bottom_left.drag = false;
            corners.bottom_right.drag = false;
        }
        _ => (),
    }
}
