use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, Draw, Drawn, IndicesChain, IndicesFromRange, IntoDrawn, LinSrgba,
    SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke, VerticesChain,
    VerticesFromRanges,
};
use crate::draw::{self, theme, Drawing};
use crate::geom::{self, Vector2};
use crate::math::BaseFloat;
use lyon::path::iterator::FlattenedIterator;
use lyon::tessellation::{StrokeOptions, StrokeTessellator};

/// Properties related to drawing an **Ellipse**.
#[derive(Clone, Debug)]
pub struct Ellipse<S = geom::scalar::Default> {
    spatial: spatial::Properties<S>,
    no_fill: bool,
    color: Option<LinSrgba>,
    stroke: Option<StrokeOptions>,
    stroke_color: Option<LinSrgba>,
    resolution: Option<usize>,
}

// Ellipse-specific methods.

impl<S> Ellipse<S>
where
    S: BaseFloat,
{
    /// Specify no fill color.
    pub fn no_fill(mut self) -> Self {
        self.no_fill = true;
        self
    }

    /// Stroke the outline with the given color.
    pub fn stroke<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<draw::properties::ColorScalar>,
    {
        self.stroke_color = Some(color.into_lin_srgba());
        self
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        let side = radius * (S::one() + S::one());
        self.w_h(side, side)
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Ellipse<S>
where
    S: BaseFloat,
{
    type Vertices = VerticesChain<VerticesFromRanges, VerticesFromRanges>;
    type Indices = IndicesChain<IndicesFromRange, IndicesFromRange>;
    fn into_drawn(self, mut draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Ellipse {
            spatial,
            no_fill,
            color,
            stroke_color,
            resolution,
            stroke,
        } = self;

        // First get the dimensions of the ellipse.
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        assert!(
            maybe_z.is_none(),
            "z dimension support for ellipse is unimplemented"
        );

        // TODO: These should probably be adjustable via Theme.
        const DEFAULT_RESOLUTION: usize = 50;
        let default_w = || S::from(100.0).unwrap();
        let default_h = || S::from(100.0).unwrap();
        let w = maybe_x.unwrap_or_else(default_w);
        let h = maybe_y.unwrap_or_else(default_h);
        let resolution = resolution.unwrap_or(DEFAULT_RESOLUTION);
        let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
        let ellipse = geom::Ellipse::new(rect, resolution);
        let close = true;

        let (fill_vdr, fill_ir, stroke_vdr, stroke_ir) = draw.drawing_context(|ctxt| {
            let draw::DrawingContext {
                mesh,
                fill_tessellator,
            } = ctxt;

            // Fill tessellation.
            let (fill_vdr, fill_ir) = if !no_fill {
                let mut builder = mesh.builder();
                let circumference = ellipse.circumference().map(|p| {
                    let p: geom::Point2<f32> = p.cast().expect("failed to cast point");
                    p.into()
                });
                let events =
                    lyon::path::iterator::FromPolyline::new(close, circumference).path_events();
                let opts = Default::default();
                let res = fill_tessellator.tessellate_path(events, &opts, &mut builder);
                if let Err(err) = res {
                    eprintln!("fill tessellation failed: {:?}", err);
                }
                let fill_vdr = builder.vertex_data_ranges();
                let fill_ir = builder.index_range();
                (fill_vdr, fill_ir)
            } else {
                let builder = mesh.builder();
                (builder.vertex_data_ranges(), builder.index_range())
            };

            // Stroke tessellation.
            let (stroke_vdr, stroke_ir) = match (stroke, stroke_color) {
                (options, color) if options.is_some() || color.is_some() => {
                    let opts = options.unwrap_or_else(Default::default);
                    let mut builder = mesh.builder();
                    let circumference = ellipse.circumference().map(|p| {
                        let p: geom::Point2<f32> = p.cast().expect("failed to cast point");
                        p.into()
                    });
                    let events =
                        lyon::path::iterator::FromPolyline::new(close, circumference).path_events();
                    let mut stroke_tessellator = StrokeTessellator::default();
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

            (fill_vdr, fill_ir, stroke_vdr, stroke_ir)
        });

        let fill_color = match fill_ir.len() == 0 {
            true => None,
            false => {
                color.or_else(|| Some(draw.theme().fill_lin_srgba(&theme::Primitive::Ellipse)))
            }
        };
        let stroke_color = match stroke_ir.len() == 0 {
            true => None,
            false => stroke_color
                .or_else(|| Some(draw.theme().stroke_lin_srgba(&theme::Primitive::Ellipse))),
        };

        let fill_vertices = VerticesFromRanges::new(fill_vdr, fill_color);
        let fill_indices = IndicesFromRange::new(fill_ir);
        let stroke_vertices = VerticesFromRanges::new(stroke_vdr, stroke_color);
        let stroke_indices = IndicesFromRange::new(stroke_ir);
        let vertices = (fill_vertices, stroke_vertices).into();
        let indices = (fill_indices, stroke_indices).into();

        (spatial, vertices, indices)
    }
}

impl<S> Default for Ellipse<S> {
    fn default() -> Self {
        let spatial = Default::default();
        let color = Default::default();
        let resolution = Default::default();
        let stroke = None;
        let stroke_color = None;
        let no_fill = false;
        Ellipse {
            no_fill,
            spatial,
            color,
            stroke_color,
            resolution,
            stroke,
        }
    }
}

impl<S> SetOrientation<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl<S> SetPosition<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Ellipse<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl<S> SetStroke for Ellipse<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        self.stroke.stroke_options_mut()
    }
}

// Primitive conversion.

impl<S> From<Ellipse<S>> for Primitive<S> {
    fn from(prim: Ellipse<S>) -> Self {
        Primitive::Ellipse(prim)
    }
}

impl<S> Into<Option<Ellipse<S>>> for Primitive<S> {
    fn into(self) -> Option<Ellipse<S>> {
        match self {
            Primitive::Ellipse(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> Drawing<'a, Ellipse<S>, S>
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
        C: IntoLinSrgba<draw::properties::ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        self.map_ty(|ty| ty.radius(radius))
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(self, resolution: usize) -> Self {
        self.map_ty(|ty| ty.resolution(resolution))
    }
}
