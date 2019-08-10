use crate::color::conv::IntoLinSrgba;
use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IndicesChain, IndicesFromRange, IntoDrawn, LinSrgba, SetColor,
    SetOrientation, SetPosition, SetStroke, VerticesChain, VerticesFromRanges,
};
use crate::draw::{self, theme, Drawing};
use crate::geom::{self, Point2};
use crate::math::BaseFloat;
use lyon::path::iterator::FlattenedIterator;
use lyon::path::PathEvent;
use lyon::tessellation::{StrokeOptions, StrokeTessellator};
use std::ops;

/// State related to drawing a **Polygon**.
#[derive(Clone, Debug)]
pub struct PolygonInit<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    no_fill: bool,
    color: Option<LinSrgba>,
    stroke_color: Option<LinSrgba>,
    stroke: Option<StrokeOptions>,
}

/// A polygon with vertices already submitted.
#[derive(Clone, Debug)]
pub struct Polygon<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    color: Option<LinSrgba>,
    stroke_color: Option<LinSrgba>,
    vertex_data_ranges: (
        draw::IntermediaryVertexDataRanges,
        draw::IntermediaryVertexDataRanges,
    ),
    index_ranges: (ops::Range<usize>, ops::Range<usize>),
    min_index: usize,
}

/// Initialised drawing state for a polygon.
pub type DrawingPolygonInit<'a, S = geom::scalar::Default> = Drawing<'a, PolygonInit<S>, S>;

/// Initialised drawing state for a polygon.
pub type DrawingPolygon<'a, S = geom::scalar::Default> = Drawing<'a, Polygon<S>, S>;

impl<S> PolygonInit<S> {
    /// Specify no fill color.
    pub fn no_fill(mut self) -> Self {
        self.no_fill = true;
        self
    }

    /// Stroke the outline with the given color.
    pub fn stroke<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color = Some(color.into_lin_srgba());
        self
    }

    /// Submit the path events to be tessellated.
    pub(crate) fn events<I>(self, ctxt: DrawingContext<S>, events: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = PathEvent>,
    {
        let DrawingContext {
            mesh,
            fill_tessellator,
            path_event_buffer,
        } = ctxt;

        path_event_buffer.clear();
        path_event_buffer.extend(events);

        // Fill tessellation.
        let (fill_vdr, fill_ir, min_index) = if !self.no_fill {
            let mut builder = mesh.builder();
            let opts = Default::default();
            let events = path_event_buffer.iter().cloned();
            let res = fill_tessellator.tessellate_path(events, &opts, &mut builder);
            if let Err(err) = res {
                eprintln!("fill tessellation failed: {:?}", err);
            }
            (
                builder.vertex_data_ranges(),
                builder.index_range(),
                builder.min_index(),
            )
        } else {
            let builder = mesh.builder();
            (
                builder.vertex_data_ranges(),
                builder.index_range(),
                builder.min_index(),
            )
        };

        // Stroke tessellation.
        let (stroke_vdr, stroke_ir) = match (self.stroke, self.stroke_color) {
            (options, color) if options.is_some() || color.is_some() => {
                let opts = options.unwrap_or_else(Default::default);
                let mut builder = mesh.builder();
                let mut stroke_tessellator = StrokeTessellator::default();
                let events = path_event_buffer.drain(..);
                let res = stroke_tessellator.tessellate_path(events, &opts, &mut builder);
                if let Err(err) = res {
                    eprintln!("stroke tessellation failed: {:?}", err);
                }
                let stroke_vdr = builder.vertex_data_ranges();
                let stroke_ir = builder.index_range();
                (stroke_vdr, stroke_ir)
            }
            _ => (Default::default(), 0..0),
        };

        path_event_buffer.clear();

        Polygon {
            position: self.position,
            orientation: self.orientation,
            color: self.color,
            stroke_color: self.stroke_color,
            vertex_data_ranges: (fill_vdr, stroke_vdr),
            index_ranges: (fill_ir, stroke_ir),
            min_index,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext<S>, points: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        let points = points.into_iter().map(|p| {
            let p: Point2<f32> = p.into().cast().expect("failed to cast point");
            p.into()
        });
        let close = true;
        let events = lyon::path::iterator::FromPolyline::new(close, points).path_events();
        self.events(ctxt, events)
    }
}

impl<'a, S> DrawingPolygonInit<'a, S>
where
    S: BaseFloat,
{
    /// Specify no fill color.
    pub fn no_fill(self) -> Self {
        self.map_ty(|ty| ty.no_fill())
    }

    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Describe the polygon with a sequence of path events.
    pub fn events<I>(self, events: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Describe the polygon with a sequence of points.
    pub fn points<I>(self, points: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }
}

impl<S> IntoDrawn<S> for Polygon<S>
where
    S: BaseFloat,
{
    type Vertices = VerticesChain<VerticesFromRanges, VerticesFromRanges>;
    type Indices = IndicesChain<IndicesFromRange, IndicesFromRange>;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polygon {
            position,
            orientation,
            color,
            stroke_color,
            vertex_data_ranges: (fill_vdr, stroke_vdr),
            index_ranges: (fill_ir, stroke_ir),
            min_index,
        } = self;

        let fill_color = match fill_ir.len() == 0 {
            true => None,
            false => {
                color.or_else(|| Some(draw.theme().fill_lin_srgba(&theme::Primitive::Polygon)))
            }
        };
        let stroke_color = match stroke_ir.len() == 0 {
            true => None,
            false => stroke_color
                .or_else(|| Some(draw.theme().stroke_lin_srgba(&theme::Primitive::Polygon))),
        };

        let fill_vertices = VerticesFromRanges::new(fill_vdr, fill_color);
        let fill_indices = IndicesFromRange::new(fill_ir, min_index);
        let stroke_vertices = VerticesFromRanges::new(stroke_vdr, stroke_color);
        let stroke_indices = IndicesFromRange::new(stroke_ir, min_index);
        let vertices = (fill_vertices, stroke_vertices).into();
        let indices = (fill_indices, stroke_indices).into();
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };

        (spatial, vertices, indices)
    }
}

impl<S> Default for PolygonInit<S> {
    fn default() -> Self {
        let position = Default::default();
        let orientation = Default::default();
        let no_fill = false;
        let color = None;
        let stroke_color = None;
        let stroke = Default::default();
        PolygonInit {
            position,
            orientation,
            no_fill,
            color,
            stroke_color,
            stroke,
        }
    }
}

impl<S> SetOrientation<S> for PolygonInit<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for PolygonInit<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> SetColor<ColorScalar> for PolygonInit<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl<S> SetStroke for PolygonInit<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.stroke)
    }
}

impl<S> SetOrientation<S> for Polygon<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Polygon<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> SetColor<ColorScalar> for Polygon<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl<S> From<PolygonInit<S>> for Primitive<S> {
    fn from(prim: PolygonInit<S>) -> Self {
        Primitive::PolygonInit(prim)
    }
}

impl<S> From<Polygon<S>> for Primitive<S> {
    fn from(prim: Polygon<S>) -> Self {
        Primitive::Polygon(prim)
    }
}

impl<S> Into<Option<PolygonInit<S>>> for Primitive<S> {
    fn into(self) -> Option<PolygonInit<S>> {
        match self {
            Primitive::PolygonInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polygon<S>>> for Primitive<S> {
    fn into(self) -> Option<Polygon<S>> {
        match self {
            Primitive::Polygon(prim) => Some(prim),
            _ => None,
        }
    }
}
