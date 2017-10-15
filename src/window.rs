use App;
use glium::{self, glutin};

pub use glium::glutin::WindowId as Id;

/// For building an OpenGL window.
///
/// Window parameters can be specified via the `window` method.
///
/// OpenGL context parameters can be specified via the `context` method.
pub struct Builder<'a, 'b> {
    app: &'a App,
    window: glutin::WindowBuilder,
    context: glutin::ContextBuilder<'b>,
}

impl<'a, 'b> Builder<'a, 'b> {
    /// Begin building a new OpenGL window.
    pub fn new(app: &'a App) -> Self {
        Builder {
            app,
            window: glutin::WindowBuilder::new(),
            context: glutin::ContextBuilder::new(),
        }
    }

    /// Build the GL window with some custom window parameters.
    pub fn window(mut self, window: glutin::WindowBuilder) -> Self {
        self.window = window;
        self
    }

    /// Build the GL window with some custom OpenGL Context parameters.
    pub fn context<'c>(self, context: glutin::ContextBuilder<'c>) -> Builder<'a, 'c> {
        let Builder { app, window, .. } = self;
        Builder { app, window, context }
    }

    /// Builds the window, inserts it into the `App`'s display map and returns the unique ID.
    pub fn build(self) -> Result<Id, glium::backend::glutin::DisplayCreationError> {
        let Builder { app, window, context } = self;
        let display = glium::Display::new(window, context, &app.events_loop)?;
        let window_id = display.gl_window().id();
        app.displays.borrow_mut().insert(window_id, display);
        Ok(window_id)
    }
}
