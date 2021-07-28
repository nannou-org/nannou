pub mod render;

use crate::color::conv::IntoLinSrgba;
use crate::draw::drawing::DrawingContext;
use crate::draw::mesh::vertex::TexCoords;
use crate::draw::primitive::path::PathEventSource;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::Drawing;
use crate::geom::Point2;
use crate::wgpu;
use lyon::path::PathEvent;
use lyon::tessellation::StrokeOptions;

/// A trait implemented for all polygon draw primitives.
pub trait SetPolygon: Sized {
    /// Access to the polygon builder parameters.
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions;

    /// Specify no fill color and in turn no fill tessellation for the polygon.
    fn no_fill(mut self) -> Self {
        self.polygon_options_mut().no_fill = true;
        self
    }

    /// Specify a color to use for stroke tessellation.
    ///
    /// Stroke tessellation will only be performed if this method or one of the `SetStroke` methods
    /// are called.
    fn stroke_color<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.polygon_options_mut().stroke_color = Some(color.into_lin_srgba());
        self
    }

    /// Specify the whole set of polygon options.
    fn polygon_options(mut self, opts: PolygonOptions) -> Self {
        *self.polygon_options_mut() = opts;
        self
    }
}

/// State related to drawing a **Polygon**.
#[derive(Clone, Debug, Default)]
pub struct PolygonInit {
    pub(crate) opts: PolygonOptions,
}

/// The set of options shared by all polygon types.
#[derive(Clone, Debug, Default)]
pub struct PolygonOptions {
    pub position: position::Properties,
    pub orientation: orientation::Properties,
    pub no_fill: bool,
    pub stroke_color: Option<LinSrgba>,
    pub color: Option<LinSrgba>,
    pub stroke: Option<StrokeOptions>,
}

/// A polygon with vertices already submitted.
#[derive(Clone, Debug)]
pub struct Polygon {
    opts: PolygonOptions,
    path_event_src: PathEventSource,
    texture_view: Option<wgpu::TextureView>,
}

/// Initialised drawing state for a polygon.
pub type DrawingPolygonInit<'a> = Drawing<'a, PolygonInit>;

/// Initialised drawing state for a polygon.
pub type DrawingPolygon<'a> = Drawing<'a, Polygon>;

impl PolygonInit {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Submit the path events to be tessellated.
    pub(crate) fn events<I>(self, ctxt: DrawingContext, events: I) -> Polygon
    where
        I: IntoIterator<Item = PathEvent>,
    {
        let DrawingContext {
            path_event_buffer, ..
        } = ctxt;
        let start = path_event_buffer.len();
        path_event_buffer.extend(events);
        let end = path_event_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::Buffered(start..end),
            texture_view: None,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext, points: I) -> Polygon
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        let points = points.into_iter().map(|p| {
            let p: Point2 = p.into();
            p.to_array().into()
        });
        let close = true;
        let events = lyon::path::iterator::FromPolyline::new(close, points);
        self.events(ctxt, events)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_colored<I, P, C>(self, ctxt: DrawingContext, points: I) -> Polygon
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        let DrawingContext {
            path_points_colored_buffer,
            ..
        } = ctxt;
        let start = path_points_colored_buffer.len();
        let points = points
            .into_iter()
            .map(|(p, c)| (p.into(), c.into_lin_srgba()));
        path_points_colored_buffer.extend(points);
        let end = path_points_colored_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::ColoredPoints {
                range: start..end,
                close: true,
            },
            texture_view: None,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_textured<I, P, T>(
        self,
        ctxt: DrawingContext,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> Polygon
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point2>,
        T: Into<TexCoords>,
    {
        let DrawingContext {
            path_points_textured_buffer,
            ..
        } = ctxt;
        let start = path_points_textured_buffer.len();
        let points = points.into_iter().map(|(p, c)| (p.into(), c.into()));
        path_points_textured_buffer.extend(points);
        let end = path_points_textured_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::TexturedPoints {
                range: start..end,
                close: true,
            },
            texture_view: Some(view.to_texture_view()),
        }
    }
}

impl<'a, T> Drawing<'a, T>
where
    T: SetPolygon + Into<Primitive>,
    Primitive: Into<Option<T>>,
{
    /// Specify no fill color and in turn no fill tessellation for the polygon.
    pub fn no_fill(self) -> Self {
        self.map_ty(|ty| ty.no_fill())
    }

    /// Specify a color to use for stroke tessellation.
    ///
    /// Stroke tessellation will only be performed if this method or one of the `SetStroke` methods
    /// are called.
    pub fn stroke_color<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke_color(color))
    }

    /// Specify the whole set of polygon options.
    pub fn polygon_options(self, opts: PolygonOptions) -> Self {
        self.map_ty(|ty| ty.polygon_options(opts))
    }
}

impl<'a> DrawingPolygonInit<'a> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Describe the polygon with a sequence of path events.
    pub fn events<I>(self, events: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Describe the polygon with a sequence of points.
    pub fn points<I>(self, points: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt, points))
    }

    /// Describe the polygon with an iterator yielding textured poings.
    pub fn points_textured<I, P, T>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point2>,
        T: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt, view, points))
    }
}

impl SetPolygon for PolygonOptions {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        self
    }
}

impl SetOrientation for PolygonInit {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl SetPosition for PolygonInit {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl SetColor<ColorScalar> for PolygonInit {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.opts.color)
    }
}

impl SetPolygon for PolygonInit {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.opts)
    }
}

impl SetStroke for PolygonInit {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.opts.stroke)
    }
}

impl SetOrientation for Polygon {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl SetPosition for Polygon {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl SetColor<ColorScalar> for Polygon {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.opts.color)
    }
}

impl From<PolygonInit> for Primitive {
    fn from(prim: PolygonInit) -> Self {
        Primitive::PolygonInit(prim)
    }
}

impl From<Polygon> for Primitive {
    fn from(prim: Polygon) -> Self {
        Primitive::Polygon(prim)
    }
}

impl Into<Option<PolygonInit>> for Primitive {
    fn into(self) -> Option<PolygonInit> {
        match self {
            Primitive::PolygonInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<Polygon>> for Primitive {
    fn into(self) -> Option<Polygon> {
        match self {
            Primitive::Polygon(prim) => Some(prim),
            _ => None,
        }
    }
}
