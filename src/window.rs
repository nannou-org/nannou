use App;
use glium::{self, glutin};
use glium::glutin::{CursorState, GlContext, MonitorId, MouseCursor};

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

/// An OpenGL window.
///
/// The `Window` acts as a wrapper around the `glium::Display` type, providing a more
/// nannou-friendly API.
pub struct Window {
    pub (super) display: glium::Display,
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
        app.windows.borrow_mut().insert(window_id, Window { display });
        Ok(window_id)
    }

    fn map_window<F>(self, map: F) -> Self
        where F: FnOnce(glutin::WindowBuilder) -> glutin::WindowBuilder,
    {
        let Builder { app, window, context } = self;
        let window = map(window);
        Builder { app, window, context }
    }

    fn map_context<F>(self, map: F) -> Self
        where F: FnOnce(glutin::ContextBuilder) -> glutin::ContextBuilder,
    {
        let Builder { app, window, context } = self;
        let context = map(context);
        Builder { app, window, context }
    }

    // Window builder methods.

    /// Requests the window to be specific dimensions pixels.
    pub fn with_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_dimensions(width, height))
    }

    /// Set the minimum dimensions in pixels for the window.
    pub fn with_min_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_min_dimensions(width, height))
    }

    /// Set the maximum dimensions in pixels for the window.
    pub fn with_max_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_max_dimensions(width, height))
    }

    /// Requests a specific title for the window.
    pub fn with_title<T>(self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.map_window(|w| w.with_title(title))
    }

    /// Sets the window fullscreen state.
    ///
    /// None means a normal window, Some(MonitorId) means a fullscreen window on that specific
    /// monitor.
    pub fn with_fullscreen(self, monitor: Option<MonitorId>) -> Self {
        self.map_window(|w| w.with_fullscreen(monitor))
    }

    /// Requests maximized mode.
    pub fn with_maximized(self, maximized: bool) -> Self {
        self.map_window(|w| w.with_maximized(maximized))
    }

    /// Sets whether the window will be initially hidden or visible.
    pub fn with_visibility(self, visible: bool) -> Self {
        self.map_window(|w| w.with_visibility(visible))
    }

    /// Sets whether the background of the window should be transparent.
    pub fn with_transparency(self, transparent: bool) -> Self {
        self.map_window(|w| w.with_transparency(transparent))
    }

    /// Sets whether the window should have a border, a title bar, etc.
    pub fn with_decorations(self, decorations: bool) -> Self {
        self.map_window(|w| w.with_decorations(decorations))
    }

    /// Enables multitouch.
    pub fn with_multitouch(self) -> Self {
        self.map_window(|w| w.with_multitouch())
    }

    // Context builder methods.

    /// Sets how the backend should choose the OpenGL API and version.
    pub fn with_gl(self, request: glutin::GlRequest) -> Self {
        self.map_context(|c| c.with_gl(request))
    }

    /// Sets the desired OpenGL context profile.
    pub fn with_gl_profile(self, profile: glutin::GlProfile) -> Self {
        self.map_context(|c| c.with_gl_profile(profile))
    }

    /// Sets the debug flag for the OpenGL context.
    ///
    /// The default value for this flag is `cfg!(debug_assertions)`, which means that it's enabled
    /// when you run `cargo build` and disabled when you run `cargo build --release`.
    pub fn with_gl_debug_flag(self, flag: bool) -> Self {
        self.map_context(|c| c.with_gl_debug_flag(flag))
    }

    /// Sets the robustness of the OpenGL context. See the docs of `Robustness`.
    pub fn with_gl_robustness(self, robustness: glutin::Robustness) -> Self {
        self.map_context(|c| c.with_gl_robustness(robustness))
    }

    /// Requests that the window has vsync enabled.
    ///
    /// By default, vsync is not enabled.
    pub fn with_vsync(self, vsync: bool) -> Self {
        self.map_context(|c| c.with_vsync(vsync))
    }

    /// Sets the multisampling level to request.
    ///
    /// A value of `0` indicates that multisampling must not be enabled.
    ///
    /// **Panics** if `samples` is not a power of 2.
    pub fn with_multisampling(self, samples: u16) -> Self {
        self.map_context(|c| c.with_multisampling(samples))
    }

    /// Sets the number of bits in the depth buffer.
    pub fn with_depth_buffer(self, bits: u8) -> Self {
        self.map_context(|c| c.with_depth_buffer(bits))
    }

    /// Sets the number of bits in the stencil buffer.
    pub fn with_stencil_buffer(self, bits: u8) -> Self {
        self.map_context(|c| c.with_stencil_buffer(bits))
    }

    /// Sets the number of bits in the color buffer.
    pub fn with_pixel_format(self, color_bits: u8, alpha_bits: u8) -> Self {
        self.map_context(|c| c.with_pixel_format(color_bits, alpha_bits))
    }

    /// Request the backend to be stereoscopic.
    pub fn with_stereoscopy(self) -> Self {
        self.map_context(|c| c.with_stereoscopy())
    }

    /// Sets whether sRGB should be enabled on the window.
    ///
    /// The default value is `false`.
    pub fn with_srgb(self, enabled: bool) -> Self {
        self.map_context(|c| c.with_srgb(enabled))
    }
}

impl Window {
    const NO_LONGER_EXISTS: &'static str = "the window no longer exists";

    // `glutin::Window` methods.

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    pub fn set_title(&self, title: &str) {
        self.display.gl_window().set_title(title);
    }

    /// Shows the window if it was hidden.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn show(&self) {
        self.display.gl_window().show()
    }

    /// Hides the window if it was visible.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn hide(&self) {
        self.display.gl_window().hide()
    }

    /// The position of the top-left hand corner of the window relative to the top-left hand corner
    /// of the desktop.
    ///
    /// Note that the top-left hand corner of the desktop is not necessarily the same as the
    /// screen. If the user uses a desktop with multiple monitors, the top-left hand corner of the
    /// desktop is the top-left hand corner of the monitor at the top-left of the desktop.
    ///
    /// The coordinates can be negative if the top-left hand corner of the window is outside of the
    /// visible screen region.
    pub fn position(&self) -> (i32, i32) {
        self.display
            .gl_window()
            .get_position()
            .expect(Self::NO_LONGER_EXISTS)
    }

    /// Modifies the position of the window.
    ///
    /// See `get_position` for more information about the returned coordinates.
    pub fn set_position(&self, x: i32, y: i32) {
        self.display.gl_window().set_position(x, y)
    }

    /// The size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. To get
    /// the dimensions of the frame buffer when calling `glViewport`, multiply with hidpi factor.
    pub fn inner_size_points(&self) -> (u32, u32) {
        self.display
            .gl_window()
            .get_inner_size_points()
            .expect(Self::NO_LONGER_EXISTS)
    }

    /// The size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These
    /// are the dimensions of the frame buffer, and the dimensions that you should use when you
    /// call glViewport.
    pub fn inner_size_pixels(&self) -> (u32, u32) {
        self.display
            .gl_window()
            .get_inner_size_pixels()
            .expect(Self::NO_LONGER_EXISTS)
    }

    /// The size of the window in pixels.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_pixels` instead.
    pub fn outer_size_pixels(&self) -> (u32, u32) {
        self.display
            .gl_window()
            .get_outer_size()
            .expect(Self::NO_LONGER_EXISTS)
    }

    /// Modifies the inner size of the window.
    ///
    /// See the `inner_size` methods for more informations about the values.
    pub fn set_inner_size_pixels(&self, width: u32, height: u32) {
        self.display.gl_window().set_inner_size(width, height)
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_cursor(&self, cursor: MouseCursor) {
        self.display.gl_window().set_cursor(cursor)
    }

    /// The ratio between the backing framebuffer resolution and the window size in screen pixels.
    ///
    /// This is typically `1.0` for a normal display, `2.0` for a retina display and higher on more
    /// modern displays.
    pub fn hidpi_factor(&self) -> f32 {
        self.display.gl_window().hidpi_factor()
    }

    /// Changes the position of the cursor in window coordinates.
    pub fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), ()> {
        self.display.gl_window().set_cursor_position(x, y)
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_cursor_state(&self, state: CursorState) -> Result<(), String> {
        self.display.gl_window().set_cursor_state(state)
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.display.gl_window().set_maximized(maximized)
    }

    /// Sets how winit handles the cursor.
    ///
    /// See the documentation of `CursorState` for details.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_fullscreen(&self, monitor: Option<MonitorId>) {
        self.display.gl_window().set_fullscreen(monitor)
    }

    /// The current monitor that the window is on or the primary monitor if nothing matches.
    pub fn current_monitor(&self) -> MonitorId {
        self.display.gl_window().get_current_monitor()
    }

    /// A unique identifier associated with this window.
    pub fn id(&self) -> Id {
        self.display.gl_window().id()
    }

    // `glutin::Context` methods.

    /// True if this context is the current one in this thread.
    pub fn is_current(&self) -> bool {
        self.display.gl_window().is_current()
    }

    /// The address of an OpenGL function.
    pub fn proc_address(&self, addr: &str) -> *const () {
        self.display.gl_window().get_proc_address(addr)
    }

    /// The OpenGL API being used.
    pub fn opengl_api(&self) -> glutin::Api {
        self.display.gl_window().get_api()
    }

    /// The pixel format of the main framebuffer of the context.
    pub fn pixel_format(&self) -> glutin::PixelFormat {
        self.display.gl_window().get_pixel_format()
    }

    // `glium::Display` methods.

    /// Rebuilds the inner `Display`'s `GlWindow` with the given window and context builders.
    ///
    /// This method ensures that the new OpenGL context will share the display lists of the
    /// original.
    pub fn rebuild(
        &self,
        window: glutin::WindowBuilder,
        context: glutin::ContextBuilder,
        app: &App,
    ) -> Result<(), glium::backend::glutin::DisplayCreationError>
    {
        self.display.rebuild(window, context, &app.events_loop)
    }

    // `glium::Context` methods.

    /// The dimensions of the framebuffer - often equivalent to the inner dimensions of a window.
    pub fn framebuffer_dimensions(&self) -> (u32, u32) {
        self.display.get_framebuffer_dimensions()
    }

    /// The OpenGL version detected by this context.
    pub fn opengl_version(&self) -> &glium::Version {
        self.display.get_opengl_version()
    }

    /// The GLSL version guaranteed to be supported.
    pub fn supported_glsl_version(&self) -> glium::Version {
        self.display.get_supported_glsl_version()
    }

    /// Returns true if the given GLSL version is supported.
    pub fn is_glsl_version_supported(&self, version: &glium::Version) -> bool {
        self.display.is_glsl_version_supported(version)
    }

    /// A string containing this GL version or release number used by this context.
    ///
    /// Vendor-specific information may follow the version number.
    pub fn opengl_version_string(&self) -> &str {
        self.display.get_opengl_version_string()
    }

    /// A string containing the company responsible for this GL implementation.
    pub fn opengl_vendor_string(&self) -> &str {
        self.display.get_opengl_vendor_string()
    }

    /// Returns a string containing the name of the GL renderer used by this context.
    ///
    /// This name is typically specific to a particular configuration of a hardware platform.
    pub fn opengl_renderer_string(&self) -> &str {
        self.display.get_opengl_renderer_string()
    }

    /// Returns true if the context is in debug mode.
    ///
    /// Debug mode may provide additional error and performance issue reporting functionality.
    pub fn is_debug(&self) -> bool {
        self.display.is_debug()
    }

    /// Returns true if the context is in "forward-compatible" mode.
    ///
    /// Forward-compatible mode means that no deprecated functionality will be supported.
    pub fn is_forward_compatible(&self) -> bool {
        self.display.is_forward_compatible()
    }

    /// This context's OpenGL profile if available.
    ///
    /// The context profile is available from OpenGL 3.2 onwards.
    ///
    /// Returns `None` if not supported.
    pub fn opengl_profile(&self) -> Option<glium::Profile> {
        self.display.get_opengl_profile()
    }

    /// Returns true if out-of-bound buffer access from the GPU side (inside a program) cannot
    /// result in a crash.
    ///
    /// You should take extra care if `is_robust` returns false.
    pub fn is_robust(&self) -> bool {
        self.display.is_robust()
    }

    /// Returns true if context loss is possible.
    pub fn is_context_loss_possible(&self) -> bool {
        self.display.is_context_loss_possible()
    }

    /// Returnstrue if the context has been lost and needs to be recreated.
    ///
    /// If it has been determined that the context has been lost before, then the function
    /// immediately returns true. Otherwise calls glGetGraphicsResetStatus. If this function is not
    /// available, returns false.
    pub fn is_context_lost(&self) -> bool {
        self.display.is_context_lost()
    }

    /// The behaviour when the current OpenGL context is changed.
    ///
    /// The most common value is `Flush`. In order to get `None` you must explicitly request it
    /// during creation.
    pub fn release_behavior(&self) -> glium::backend::ReleaseBehavior {
        self.display.get_release_behavior()
    }

    /// Returns the maximum value that can be used for anisotropic filtering.
    ///
    /// Returns `None` if the hardware doesn't support it.
    pub fn max_anisotropy_support(&self) -> Option<u16> {
        self.display.get_max_anisotropy_support()
    }

    /// The maximum dimensions of the viewport.
    ///
    /// Glium will panic if you request a larger viewport than this when drawing.
    pub fn max_viewport_dimensions(&self) -> (u32, u32) {
        self.display.get_max_viewport_dimensions()
    }

    /// Provides a hint to the OpenGL implementation that it may free internal resources associated
    /// with its shader compiler.
    ///
    /// This method is a no-op if it's not available in the implementation.
    pub fn release_shader_compiler(&self) {
        self.display.release_shader_compiler()
    }

    /// An estimate of the amount  of video memory available in bytes.
    ///
    /// `None` if no estimate is available.
    pub fn free_video_memory_bytes(&self) -> Option<usize> {
        self.display.get_free_video_memory()
    }

    /// Reads the content of the front buffer.
    ///
    /// You will only see the data that has finished being drawn.
    ///
    /// This function can return any type that implements `Texture2dDataSink<(u8, u8, u8, u8)>`.
    pub fn read_front_buffer<T>(&self) -> T
    where
        T: glium::texture::Texture2dDataSink<(u8, u8, u8, u8)>,
    {
        self.display.read_front_buffer()
    }

    /// Execute an arbitrary closure with the OpenGL context active. Useful if another component
    /// needs to directly manipulate OpenGL state.
    ///
    ///
    /// **If `action` maniuplates any OpenGL state, it must be restored before `action`
    /// completes.**
    pub unsafe fn exec_in_context<'a, T, F>(&self, action: F) -> T
    where
        T: Send + 'static,
        F: FnOnce() -> T + 'a,
    {
        self.display.exec_in_context(action)
    }

    /// Asserts that there are no OpenGL errors pending.
    ///
    /// This function should be used in tests.
    pub fn assert_no_error(&self, user_msg: Option<&str>) {
        self.display.assert_no_error(user_msg)
    }

    /// Inserts a debugging string in the commands queue. If you use an OpenGL debugger, you will
    /// be able to see that string.
    ///
    /// This is helpful to understand where you are when you have big applications.
    ///
    /// Returns `Err` if the backend doesn't support this functionnality. You can choose whether to
    /// call .unwrap() if you want to make sure that it works, or .ok() if you don't care.
    pub fn insert_debug_marker(&self, marker: &str) -> Result<(), ()> {
        self.display.insert_debug_marker(marker)
    }

    /// Same as `insert_debug_marker`, except that if you don't compile with `debug_assertions` it
    /// is a no-op and returns `Ok`.
    pub fn debug_insert_debug_marker(&self, marker: &str) -> Result<(), ()> {
        self.display.debug_insert_debug_marker(marker)
    }

    // Using the following methods may possibly break nannou's API and should be avoided if
    // possible, however they are provided for flexibility.

    /// Returns a reference to the inner `glium::Display`.
    ///
    /// **Note:** using this method (or more spefically, some methods on `Display`) may break the
    /// nannou API. It should be avoided if possible, but is provided for flexibility in case it is
    /// needed or in case nannou's API does not suffice.
    pub fn inner_glium_display(&self) -> &glium::Display {
        &self.display
    }
}
