//! The `glium` backend for rendering the contents of a **Draw**'s mesh.

use draw;
use glium;
use math::{BaseFloat, NumCast};
use std::error::Error;
use std::fmt;

/// The `Vertex` type passed to the vertex shader.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    // /// The mode with which the `Vertex` will be drawn within the fragment shader.
    // ///
    // /// `0` for rendering text.
    // /// `1` for rendering an image.
    // /// `2` for rendering non-textured 2D geometry.
    // ///
    // /// If any other value is given, the fragment shader will not output any color.
    // pub mode: u32,
    
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, -1.0, 0.0] is the leftmost, bottom position of the display.
    /// [1.0, 1.0, 0.0] is the rightmost, top position of the display.
    pub position: [f32; 3],
    /// A color associated with the `Vertex`.
    ///
    /// The way that the color is used depends on the `mode`.
    pub color: [f32; 4],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, bottom position of the texture.
    /// [1.0, 1.0] is the rightmost, top position of the texture.
    pub tex_coords: [f32; 2],
}

// /// Draw text from the text cache texture `tex` in the fragment shader.
// pub const MODE_TEXT: u32 = 0;
// /// Draw an image from the texture at `tex` in the fragment shader.
// pub const MODE_IMAGE: u32 = 1;
// /// Ignore `tex` and draw simple, colored 2D geometry.
// pub const MODE_GEOMETRY: u32 = 2;

#[allow(unsafe_code)]
mod vertex_impl {
    use super::Vertex;
    implement_vertex!(Vertex, position, color, tex_coords);
}

impl Vertex {
    /// Create a vertex from the given mesh vertex.
    pub fn from_mesh_vertex<S>(
        v: draw::mesh::Vertex<S>,
        framebuffer_width: f32,
        framebuffer_height: f32,
        dpi_factor: f32,
    ) -> Self
    where
        S: BaseFloat,
    {
        let point = v.point();
        let x_f: f32 = NumCast::from(point.x).unwrap();
        let y_f: f32 = NumCast::from(point.y).unwrap();
        let z_f: f32 = NumCast::from(point.z).unwrap();
        // Map coords from (-fb_dim, +fb_dim) to (-1.0, 1.0)
        let x = 2.0 * x_f * dpi_factor / framebuffer_width;
        let y = 2.0 * y_f * dpi_factor / framebuffer_height;
        let z = 2.0 * z_f * dpi_factor / framebuffer_height;
        let tex_x = NumCast::from(v.tex_coords.x).unwrap();
        let tex_y = NumCast::from(v.tex_coords.y).unwrap();
        let position = [x, y, z];
        let color = [v.color.red, v.color.green, v.color.blue, v.color.alpha];
        let tex_coords = [tex_x, tex_y];
        Vertex { position, color, tex_coords }
    }
}

// GLSL (ported from conrod)

/// The vertex shader used within the `glium::Program` for OpenGL.
pub const VERTEX_SHADER_120: &'static str = "
    #version 120

    attribute vec3 position;
    attribute vec4 color;
    attribute vec2 tex_coords;
    // attribute float mode;

    varying vec4 v_color;
    varying vec2 v_tex_coords;
    // varying float v_mode;

    void main() {
        gl_Position = vec4(position, 1.0);
        v_color = color;
        v_tex_coords = tex_coords;
        // v_mode = mode;
    }
";

/// The fragment shader used within the `glium::Program` for OpenGL.
pub const FRAGMENT_SHADER_120: &'static str = "
    #version 120
    // uniform sampler2D tex;

    varying vec2 v_tex_coords;
    varying vec4 v_color;
    // varying float v_mode;

    void main() {
        // // Text
        // if (v_mode == 0.0) {
        //     gl_FragColor = v_color * vec4(1.0, 1.0, 1.0, texture2D(tex, v_tex_coords).r);

        // // Image
        // } else if (v_mode == 1.0) {
        //     gl_FragColor = texture2D(tex, v_tex_coords);

        // // 2D Geometry
        // } else if (v_mode == 2.0) {
        //     gl_FragColor = v_color;
        // }

        gl_FragColor = v_color;
    }
";

/// The vertex shader used within the `glium::Program` for OpenGL.
pub const VERTEX_SHADER_140: &'static str = "
    #version 140

    in vec3 position;
    in vec4 color;
    in vec2 tex_coords;
    // in uint mode;

    out vec4 v_color;
    out vec2 v_tex_coords;
    // flat out uint v_mode;

    void main() {
        gl_Position = vec4(position, 1.0);
        v_color = color;
        v_tex_coords = tex_coords;
        // v_mode = mode;
    }
";

/// The fragment shader used within the `glium::Program` for OpenGL.
pub const FRAGMENT_SHADER_140: &'static str = "
    #version 140
    // uniform sampler2D tex;

    in vec4 v_color;
    in vec2 v_tex_coords;
    // flat in uint v_mode;

    out vec4 f_color;

    void main() {
        // // Text
        // if (v_mode == uint(0)) {
        //     f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

        // // Image
        // } else if (v_mode == uint(1)) {
        //     f_color = texture(tex, v_tex_coords);

        // // 2D Geometry
        // } else if (v_mode == uint(2)) {
        //     f_color = v_color;
        // }

        f_color = v_color;
    }
";

/// The vertex shader used within the `glium::Program` for OpenGL ES.
pub const VERTEX_SHADER_300_ES: &'static str = "
    #version 300 es
    precision mediump float;

    in vec3 position;
    in vec4 color;
    in vec2 tex_coords;
    // in uint mode;

    out vec4 v_color;
    out vec2 v_tex_coords;
    // flat out uint v_mode;

    void main() {
        gl_Position = vec4(position, 1.0);
        v_color = color;
        v_tex_coords = tex_coords;
        // v_mode = mode;
    }
";

/// The fragment shader used within the `glium::Program` for OpenGL ES.
pub const FRAGMENT_SHADER_300_ES: &'static str = "
    #version 300 es
    precision mediump float;
    // uniform sampler2D tex;

    in vec4 v_color;
    in vec2 v_tex_coords;
    // flat in uint v_mode;

    out vec4 f_color;

    void main() {
        // // Text
        // if (v_mode == uint(0)) {
        //     f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

        // // Image
        // } else if (v_mode == uint(1)) {
        //     f_color = texture(tex, v_tex_coords);

        // // 2D Geometry
        // } else if (v_mode == uint(2)) {
        //     f_color = v_color;
        // }

        f_color = v_color;
    }
";

/// Construct the glium shader program that can be used to render `Vertex`es.
pub fn program<F>(facade: &F) -> Result<glium::Program, glium::program::ProgramChooserCreationError>
    where F: glium::backend::Facade,
{
    program!(facade,
             120 => { vertex: VERTEX_SHADER_120, fragment: FRAGMENT_SHADER_120 },
             140 => { vertex: VERTEX_SHADER_140, fragment: FRAGMENT_SHADER_140 },
             300 es => { vertex: VERTEX_SHADER_300_ES, fragment: FRAGMENT_SHADER_300_ES })
}

/// Default glium `DrawParameters` with alpha blending enabled.
pub fn draw_parameters() -> glium::DrawParameters<'static> {
    let blend = glium::Blend::alpha_blending();
    glium::DrawParameters { multisampling: true, blend: blend, ..Default::default() }
}

/// Errors that might occur during a call to `Renderer::draw`.
#[derive(Debug)]
pub enum RendererDrawError {
    BufferCreation(BufferCreationError),
    Draw(glium::DrawError),
}

#[derive(Debug)]
pub enum BufferCreationError {
    Vertex(glium::vertex::BufferCreationError),
    Index(glium::index::BufferCreationError),
}

/// A type used for rendering a **nannou::draw::Mesh** to an OpenGL surface via glium.
pub struct Renderer {
    program: glium::Program,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Renderer {
    /// Build a new **nannou::draw::backend::glium::Renderer**.
    pub fn new<F>(facade: &F) -> Result<Self, glium::program::ProgramChooserCreationError>
    where
        F: glium::backend::Facade,
    {
        let program = program(facade)?;
        let vertices = vec![];
        let indices = vec![];
        Ok(Renderer { program, vertices, indices })
    }

    /// Draw the given mesh to the given glium surface.
    pub fn draw<S, F, T>(
        &mut self,
        draw: &draw::Draw<S>,
        facade: &F,
        dpi_factor: f32,
        surface: &mut T,
    ) -> Result<(), RendererDrawError>
    where
        S: BaseFloat,
        F: glium::backend::Facade,
        T: glium::Surface,
    {
        // If some background color was given, clear the screen with it.
        if let Some(color) = draw.state.borrow().background_color {
            surface.clear_color(color.red, color.green, color.blue, color.alpha);
        }

        // Create the vertex and index buffers.
        self.vertices.clear();
        self.indices.clear();
        let (w, h) = facade.get_context().get_framebuffer_dimensions();
        let map_vertex = |v| Vertex::from_mesh_vertex(v, w as _, h as _, dpi_factor);
        self.vertices.extend(draw.raw_vertices().map(map_vertex));
        self.indices.extend(draw.mesh().indices().iter().map(|&u| u as u32));
        let index_prim = glium::index::PrimitiveType::TrianglesList;
        let vertex_buffer = glium::VertexBuffer::new(facade, &self.vertices[..])?;
        let index_buffer = glium::IndexBuffer::new(facade, index_prim, &self.indices[..])?;

        // Create the draw parameters.
        let draw_params = draw_parameters();

        // Create the uniforms.
        let uniforms = uniform!{};

        // Draw to the given surface.
        surface.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &draw_params)?;

        Ok(())
    }
}

impl Error for BufferCreationError {
    fn description(&self) -> &str {
        match *self {
            BufferCreationError::Vertex(ref err) => err.description(),
            BufferCreationError::Index(ref err) => err.description(),
        }
    }
}

impl Error for RendererDrawError {
    fn description(&self) -> &str {
        match *self {
            RendererDrawError::BufferCreation(ref err) => err.description(),
            RendererDrawError::Draw(ref err) => err.description(),
        }
    }
}

impl fmt::Display for BufferCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for RendererDrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<glium::vertex::BufferCreationError> for BufferCreationError {
    fn from(err: glium::vertex::BufferCreationError) -> Self {
        BufferCreationError::Vertex(err)
    }
}

impl From<glium::index::BufferCreationError> for BufferCreationError {
    fn from(err: glium::index::BufferCreationError) -> Self {
        BufferCreationError::Index(err)
    }
}

impl From<BufferCreationError> for RendererDrawError {
    fn from(err: BufferCreationError) -> Self {
        RendererDrawError::BufferCreation(err)
    }
}

impl From<glium::DrawError> for RendererDrawError {
    fn from(err: glium::DrawError) -> Self {
        RendererDrawError::Draw(err)
    }
}

impl From<glium::vertex::BufferCreationError> for RendererDrawError {
    fn from(err: glium::vertex::BufferCreationError) -> Self {
        RendererDrawError::BufferCreation(err.into())
    }
}

impl From<glium::index::BufferCreationError> for RendererDrawError {
    fn from(err: glium::index::BufferCreationError) -> Self {
        RendererDrawError::BufferCreation(err.into())
    }
}
