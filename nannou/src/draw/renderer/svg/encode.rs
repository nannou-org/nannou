use crate::draw::primitive::path::Options;
use crate::draw::properties::LinSrgba;
use crate::glam::{Mat4, Vec3};
use crate::lyon::math::Point;
use lyon::lyon_tessellation::{LineCap, LineJoin};
use lyon::path::{Event, FillRule, PathEvent};
use nannou_core::color::{Alpha, Srgb};
use nannou_core::geom::Rect;
use svg::node::element::path::Position::Absolute;
use svg::node::element::path::{Command, Data, Parameters};
use svg::node::element::{Group, Path, Rectangle};
use svg::{Document, Node};

pub fn svg_document(dims: Rect, elements: Group, background: Option<LinSrgba>) -> Document {
    let dims = dims.absolute();
    let corner = dims.bottom_left(); // smallest x and y
    let size = dims.wh();
    let y_offset = dims.y() * 2.0;

    let mut document = Document::new().set("viewBox", (corner.x, corner.y, size.x, size.y));

    if let Some(color) = background {
        document.append(
            Rectangle::new()
                .set("fill", convert_color(color))
                .set("x", corner.x)
                .set("y", corner.y)
                .set("width", "100%")
                .set("height", "100%"),
        );
    }

    document.append(elements.set(
        "transform",
        format!("translate(0 {}) scale(1 -1)", y_offset), // in svg, +y is down; in nannou, +y is up
    ));

    document
}

pub fn render_path<I>(
    svg: &mut Group,
    events: I,
    transform: Mat4,
    color: LinSrgba,
    options: Options,
) where
    I: Iterator<Item = PathEvent>,
{
    let path_data = lyon_to_svg_path_data(events, transform);
    render_path_data(svg, path_data, color, options);
}

pub fn render_path_data(svg: &mut Group, path_data: Data, color: LinSrgba, options: Options) {
    svg.append(path_options(color, options).set("d", path_data));
}

pub fn lyon_to_svg_path_data<I>(events: I, transform: Mat4) -> Data
where
    I: Iterator<Item = PathEvent>,
{
    let param = |pt: Point| -> Parameters {
        let pt: Vec<_> = transform
            .transform_point3(Vec3::new(pt.x, pt.y, 0.0))
            .truncate()
            .to_array()
            .into();
        pt.into()
    };

    let params = |points: &[Point]| -> Parameters {
        let mut params = Vec::with_capacity(points.len() * 2);
        for pt in points {
            let pt = transform.transform_point3(Vec3::new(pt.x, pt.y, 0.0));
            params.push(pt.x);
            params.push(pt.y);
        }
        params.into()
    };

    let mut data = Vec::with_capacity(events.size_hint().0);

    // follows lyon::FromPolyline convention that the previous Line.to == current Line.from
    for event in events {
        match event {
            Event::Begin { at } => data.push(Command::Move(Absolute, param(at))),
            Event::Line { to, .. } => data.push(Command::Line(Absolute, param(to))),
            Event::Quadratic { ctrl, to, .. } => {
                data.push(Command::QuadraticCurve(Absolute, params(&[ctrl, to])))
            }
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => data.push(Command::CubicCurve(Absolute, params(&[ctrl1, ctrl2, to]))),
            Event::End { close, .. } if close => data.push(Command::Close),
            Event::End { .. } => {}
        }
    }

    data.into()
}

fn path_options(color: LinSrgba, options: Options) -> Path {
    match options {
        Options::Fill(options) => Path::new()
            .set("stroke", "none")
            .set("fill", convert_color(color))
            .set(
                "fill-rule",
                match options.fill_rule {
                    FillRule::EvenOdd => "evenodd",
                    FillRule::NonZero => "nonzero",
                },
            ),
        Options::Stroke(options) => {
            if options.start_cap != options.end_cap {
                unimplemented!();
            }
            Path::new()
                .set("fill", "none")
                .set("stroke", convert_color(color))
                .set(
                    "stroke-linecap",
                    match options.start_cap {
                        LineCap::Butt => "butt",
                        LineCap::Square => "square",
                        LineCap::Round => "round",
                    },
                )
                .set(
                    "stroke-linejoin",
                    match options.line_join {
                        LineJoin::Miter => "miter",
                        LineJoin::MiterClip => "miter-clip",
                        LineJoin::Round => "round",
                        LineJoin::Bevel => "bevel",
                    },
                )
                .set("stroke-width", options.line_width)
                .set("stroke-miterlimit", options.miter_limit)
        }
    }
}

fn convert_color(color: LinSrgba) -> String {
    let color: Alpha<Srgb<u8>, f32> = color.into_encoding().into_format();

    format!(
        "rgba({}, {}, {}, {:.3})",
        color.red, color.green, color.blue, color.alpha
    )
}
