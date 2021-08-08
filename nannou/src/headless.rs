use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    app, draw, event, frame, geom,
    prelude::{DurationF64, UVec2},
    state, wgpu, window,
};

pub struct App {
    window: Window,
    pub duration: state::Time,
    pub time: f32,

    draw: RefCell<Draw>,
    exit: Cell<bool>,
    frame_count: u64,
}

impl App {
    fn new(window: Window) -> Self {
        let draw = RefCell::new(Draw::default());
        let duration = state::Time::default();
        let time = duration.since_start.secs() as _;

        Self {
            window,
            duration,
            time,
            draw,
            exit: Cell::new(false),
            frame_count: 0,
        }
    }

    pub fn quit(&self) {
        self.exit.set(true);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_rect(&self) -> geom::Rect {
        self.window().rect()
    }

    pub fn draw(&self) -> Draw {
        let draw = self.draw.borrow_mut();
        draw.reset();
        draw.clone()
    }

    pub fn elapsed_frames(&self) -> u64 {
        self.frame_count
    }

    pub fn fps(&self) -> f32 {
        self.duration.updates_per_second()
    }

    pub fn exe_name(&self) -> std::io::Result<String> {
        let string = std::env::current_exe()?
            .file_stem()
            .expect("exe path contained no file stem")
            .to_string_lossy()
            .to_string();
        Ok(string)
    }
}

#[derive(Clone, Default)]
pub struct Draw {
    inner: draw::Draw,
}

impl Draw {
    pub fn to_frame(
        &self,
        app: &App,
        frame: &frame::Frame,
    ) -> Result<(), draw::renderer::DrawError> {
        let scale_factor = 1.0;
        let mut renderer = app.window.renderer.borrow_mut();
        renderer.render_to_frame(app.window.device(), self, scale_factor, frame);
        Ok(())
    }
}

impl Deref for Draw {
    type Target = draw::Draw;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type ModelFn<Model> = fn(&App) -> Model;

pub type UpdateFn<Model> = fn(&App, &mut Model, event::Update);

pub type ViewFn<Model> = fn(&App, &Model, frame::Frame);

pub type ExitFn<Model> = fn(&App, Model);

pub type SketchViewFn = fn(&App, frame::Frame);

pub(crate) enum View<Model = ()> {
    /// A view function allows for viewing the user's model.
    WithModel(ViewFn<Model>),
    // A **Simple** view function does not require a user **Model**. Simpler to get started.
    Sketch(SketchViewFn),
}

pub struct Window {
    rect: geom::Rect,
    pub(crate) device_queue_pair: Arc<wgpu::DeviceQueuePair>,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) renderer: RefCell<draw::Renderer>,
    pub(crate) frame_data: window::FrameData,
    pub(crate) frame_count: u64,
}

impl Window {
    pub fn rect(&self) -> geom::Rect {
        self.rect
    }
    // same as nannou::window::Window::capture_frame
    pub fn capture_frame<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        self.capture_frame_inner(path.as_ref());
    }

    fn capture_frame_inner(&self, path: &Path) {
        // If the parent directory does not exist, create it.
        let dir = path.parent().expect("capture_frame path has no directory");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("failed to create `capture_frame` directory");
        }

        let mut capture_next_frame_path = self
            .frame_data
            .capture
            .next_frame_path
            .lock()
            .expect("failed to lock `capture_next_frame_path`");
        *capture_next_frame_path = Some(path.to_path_buf());
    }

    pub fn await_capture_frame_jobs(
        &self,
    ) -> Result<(), wgpu::TextureCapturerAwaitWorkerTimeout<()>> {
        let capture_data = &self.frame_data.capture;
        let device = self.device();
        return capture_data.texture_capturer.await_active_snapshots(device);
    }

    pub fn device(&self) -> &wgpu::Device {
        self.device_queue_pair.device()
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.await_capture_frame_jobs().is_err() {
            eprintln!("timed out while waiting for capture jobs to complete")
        }
    }
}

pub struct Builder<M = ()> {
    model: ModelFn<M>,
    update: Option<UpdateFn<M>>,
    view: Option<View<M>>,
    exit: Option<ExitFn<M>>,
    size: Option<UVec2>,
    capture_frame_timeout: Option<Duration>,
    max_capture_frame_jobs: u32,
    backends: wgpu::BackendBit,
    power_preference: wgpu::PowerPreference,
    device_desc: wgpu::DeviceDescriptor<'static>,
    msaa_samples: u32,
}

pub struct SketchBuilder {
    builder: Builder<()>,
}

impl<M> Builder<M>
where
    M: 'static,
{
    pub const DEFAULT_BACKENDS: wgpu::BackendBit = wgpu::DEFAULT_BACKENDS;
    pub const DEFAULT_CAPTURE_FRAME_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_POWER_PREFERENCE: wgpu::PowerPreference = wgpu::DEFAULT_POWER_PREFERENCE;

    pub fn new(model: ModelFn<M>) -> Self {
        Self {
            model,
            update: None,
            view: None,
            exit: None,
            size: None,
            capture_frame_timeout: Some(Self::DEFAULT_CAPTURE_FRAME_TIMEOUT),
            max_capture_frame_jobs: num_cpus::get() as u32,
            backends: Self::DEFAULT_BACKENDS,
            power_preference: Self::DEFAULT_POWER_PREFERENCE,
            msaa_samples: frame::Frame::DEFAULT_MSAA_SAMPLES,
            device_desc: wgpu::default_device_descriptor(),
        }
    }

    pub fn view(mut self, view: ViewFn<M>) -> Self {
        self.view = Some(View::WithModel(view));
        self
    }
    pub fn update(mut self, update: UpdateFn<M>) -> Self {
        self.update = Some(update);
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Some(UVec2::new(width, height));
        self
    }

    pub fn run(self) {
        let Self {
            model,
            update,
            view,
            exit,
            size,
            capture_frame_timeout,
            max_capture_frame_jobs,
            backends,
            power_preference,
            device_desc,
            msaa_samples,
        } = self;

        let size = size.expect("No size given");

        let window = {
            let device_queue_pair = {
                let instance = wgpu::Instance::new(backends);
                let adapters = wgpu::AdapterMap::default();

                let request_adapter_opts = wgpu::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                };

                let adapter = adapters
                    .get_or_request(request_adapter_opts, &instance)
                    .ok_or(window::BuildError::NoAvailableAdapter)
                    .unwrap();

                adapter.get_or_request_device(device_desc)
            };

            let format = frame::Frame::TEXTURE_FORMAT;
            let frame_data = {
                let render = frame::RenderData::new(
                    device_queue_pair.device(),
                    size.into(),
                    format,
                    msaa_samples,
                );
                let capture =
                    frame::CaptureData::new(max_capture_frame_jobs, capture_frame_timeout);

                window::FrameData { render, capture }
            };

            let renderer = RefCell::new(draw::RendererBuilder::new().build(
                device_queue_pair.device(),
                size.into(),
                1.0,
                msaa_samples,
                format,
            ));

            let rect = geom::Rect::from_wh(size.as_f32());

            Window {
                frame_data,
                device_queue_pair,
                rect,
                format,
                renderer,
                frame_count: 0,
            }
        };

        let app = App::new(window);
        let model = (model)(&app);

        run_loop(app, model, update, view, exit);
    }
}

fn default_model(_: &App) -> () {
    ()
}

impl SketchBuilder {
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.builder = self.builder.size(width, height);
        self
    }

    pub fn run(self) {
        self.builder.run()
    }
}
impl Builder<()> {
    /// Shorthand for building a simple app that has no model, handles no events and simply draws
    /// to a single window.
    ///
    /// This is useful for late night hack sessions where you just don't care about all that other
    /// stuff, you just want to play around with some ideas or make something pretty.
    pub fn sketch(view: SketchViewFn) -> SketchBuilder {
        let mut builder = Builder::new(default_model);
        builder.view = Some(View::Sketch(view));
        SketchBuilder { builder }
    }
}

fn run_loop<M>(
    mut app: App,
    model: M,
    update_fn: Option<UpdateFn<M>>,
    view_fn: Option<View<M>>,
    exit_fn: Option<ExitFn<M>>,
) where
    M: 'static,
{
    let loop_start = Instant::now();
    let mut model = Some(model);

    let mut loop_state = app::LoopState {
        updates_since_event: 0,
        loop_start,
        last_update: loop_start,
        total_updates: 0,
    };

    while !app.exit.get() {
        if let Some(model) = model.as_mut() {
            let now = Instant::now();
            apply_update(&mut app, model, update_fn, &mut loop_state, now);
        }

        let window = &app.window;
        app.frame_count += 1;

        if let Some(model) = model.as_ref() {
            let raw_frame = frame::RawFrame::new_fake(
                window.device_queue_pair.clone(),
                app.frame_count,
                window.format,
                window.rect,
            );

            let frame_data = &window.frame_data;

            match view_fn {
                Some(View::WithModel(view)) => {
                    let frame =
                        frame::Frame::new_empty(raw_frame, &frame_data.render, &frame_data.capture);
                    view(&app, &model, frame);
                }
                Some(View::Sketch(view)) => {
                    let frame =
                        frame::Frame::new_empty(raw_frame, &frame_data.render, &frame_data.capture);
                    view(&app, frame);
                }
                None => raw_frame.submit(),
            }
        }
    }
    if let Some(model) = model.take() {
        if let Some(exit_fn) = exit_fn {
            exit_fn(&app, model);
        }
    }
}

fn apply_update<M>(
    app: &mut App,
    model: &mut M,
    update_fn: Option<UpdateFn<M>>,
    loop_state: &mut app::LoopState,
    now: Instant,
) where
    M: 'static,
{
    let since_last = now.duration_since(loop_state.last_update);
    let since_start = now.duration_since(loop_state.loop_start);
    app.duration.since_prev_update = since_last;
    app.duration.since_start = since_start;
    app.time = since_start.secs() as _;
    let update = crate::event::Update {
        since_start,
        since_last,
    };
    if let Some(update_fn) = update_fn {
        update_fn(app, model, update);
    }
    loop_state.last_update = now;
    loop_state.total_updates += 1;
    loop_state.updates_since_event += 1;
}
