use WindowId;

pub struct Draw {
    /// A list of commands that describe how to draw the frames for each window.
    pub commands: Vec<(WindowId, Command)>,
}

pub enum Command {
    Clear {
        rect: Option<&glium::Rect>,
        color: Option<(f32, f32, f32, f32)>,
        color_srgb: bool,
        depth: Option<f32>,
        stencil: Option<i32>,
    },

    Draw {
        vertex_buffer: V,
        index_buffer: I,
        program: &glium::Program,
        uniforms: &U,
        draw_parameters: &glium::DrawParameters,
    },
}
