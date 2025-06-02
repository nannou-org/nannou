//! A small GPU compute shader demonstration.
//!
//! Here we use a compute shader to calculate the amplitude of `OSCILLATOR_COUNT` number of
//! oscillators. The oscillator amplitudes are then laid out across the screen using rectangles
//! with a gray value equal to the amplitude. Real-time interaction is demonstrated by providing
//! access to time, frequency (mouse `x`) and the number of oscillators via uniform data.

use nannou::prelude::*;
use nannou::wgpu::BufferInitDescriptor;
use std::sync::{Arc, Mutex};

struct Model {
    compute: Compute,
    window: Entity,
    oscillators: Arc<Mutex<Vec<f32>>>,
    task: Option<Task<()>>,
}

struct Compute {
    oscillator_buffer: wgpu::Buffer,
    oscillator_buffer_size: wgpu::BufferAddress,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniforms {
    time: f32,
    freq: f32,
    oscillator_count: u32,
}

const OSCILLATOR_COUNT: u32 = 128;

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let w_id = app
        .new_window()
        .primary()
        .size(1440, 512)
        .view(view)
        .build();
    let window = app.window(w_id);
    let device = window.device();

    // Create the compute shader module.
    let cs_desc = wgpu::include_wgsl!("shaders/cs.wgsl");
    let cs_mod = device.create_shader_module(cs_desc);

    // Create the buffer that will store the result of our compute operation.
    let oscillator_buffer_size =
        (OSCILLATOR_COUNT as usize * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
    let oscillator_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oscillators"),
        size: oscillator_buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Create the buffer that will store time.
    let uniforms = create_uniforms(app.time(), app.mouse().x, window.rect());
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform-buffer"),
        contents: uniforms_bytes,
        usage,
    });

    // Create the bind group and pipeline.
    let bind_group_layout = create_bind_group_layout(&device);
    let bind_group = create_bind_group(
        &device,
        &bind_group_layout,
        &oscillator_buffer,
        oscillator_buffer_size,
        &uniform_buffer,
    );
    let pipeline_layout = create_pipeline_layout(&device, &bind_group_layout);
    let pipeline = create_compute_pipeline(&device, &pipeline_layout, &cs_mod);

    let compute = Compute {
        oscillator_buffer,
        oscillator_buffer_size,
        uniform_buffer,
        bind_group,
        pipeline,
    };

    // The vector that we will write oscillator values to.
    let oscillators = Arc::new(Mutex::new(vec![0.0; OSCILLATOR_COUNT as usize]));

    Model {
        compute,
        window: w_id,
        oscillators,
        task: None,
    }
}

fn update(app: &App, model: &mut Model) {
    let window = app.main_window();
    let device = window.device();
    let win_rect = window.rect();
    let compute = &mut model.compute;

    if let Some(task) = &mut model.task {
        if let Some(_) = block_on(future::poll_once(task)) {
            model.task = None;
        } else {
            return;
        }
    }

    // The buffer into which we'll read some data.
    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read-oscillators"),
        size: compute.oscillator_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // An update for the uniform buffer with the current time.
    let uniforms = create_uniforms(app.time(), app.mouse().x, win_rect);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform-data-transfer"),
        contents: uniforms_bytes,
        usage,
    });

    // The encoder we'll use to encode the compute pass.
    let desc = wgpu::CommandEncoderDescriptor {
        label: Some("oscillator-compute"),
    };
    let mut encoder = device.create_command_encoder(&desc);
    encoder.copy_buffer_to_buffer(
        &new_uniform_buffer,
        0,
        &compute.uniform_buffer,
        0,
        uniforms_size,
    );
    {
        let pass_desc = wgpu::ComputePassDescriptor {
            label: Some("nannou-wgpu_compute_shader-compute_pass"),
            timestamp_writes: None,
        };
        let mut cpass = encoder.begin_compute_pass(&pass_desc);
        cpass.set_pipeline(&compute.pipeline);
        cpass.set_bind_group(0, &compute.bind_group, &[]);
        cpass.dispatch_workgroups(OSCILLATOR_COUNT as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        &compute.oscillator_buffer,
        0,
        &read_buffer,
        0,
        compute.oscillator_buffer_size,
    );

    // Submit the compute pass to the device's queue.
    window.queue().submit(Some(encoder.finish()));

    // Spawn a future that reads the result of the compute pass.
    let oscillators = model.oscillators.clone();
    let future = async move {
        let slice = read_buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, |res| {
            tx.send(res).expect("The channel was closed");
        });
        if let Ok(_) = rx.await {
            if let Ok(mut oscillators) = oscillators.lock() {
                let bytes = &slice.get_mapped_range()[..];
                // "Cast" the slice of bytes to a slice of floats as required.
                let floats = {
                    let len = bytes.len() / std::mem::size_of::<f32>();
                    let ptr = bytes.as_ptr() as *const f32;
                    unsafe { std::slice::from_raw_parts(ptr, len) }
                };
                oscillators.copy_from_slice(floats);
            }
        }
    };

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(future);

    model.task = Some(task);
    // Check for resource cleanups and mapping callbacks.
    //
    // Note that this line is not necessary in our case, as the device we are using already gets
    // polled when nannou submits the command buffer for drawing and presentation after `view`
    // completes. If we were to use a standalone device to create our buffer and perform our
    // compute (rather than the device requested during window creation), calling `poll` regularly
    // would be a must.
    //
    // device.poll(false);
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let window = app.window(model.window);
    let rect = window.rect();

    if let Ok(oscillators) = model.oscillators.lock() {
        let w = rect.w() / OSCILLATOR_COUNT as f32;
        let h = rect.h();
        let half_w = w * 0.5;
        for (i, &osc) in oscillators.iter().enumerate() {
            let x = half_w + map_range(i as u32, 0, OSCILLATOR_COUNT, rect.left(), rect.right());
            draw.rect().w_h(w, h).x(x).color(Color::gray(osc));
        }
    }
}

fn create_uniforms(time: f32, mouse_x: f32, win_rect: geom::Rect) -> Uniforms {
    let freq = map_range(
        mouse_x,
        win_rect.left(),
        win_rect.right(),
        0.0,
        win_rect.w(),
    );
    let oscillator_count = OSCILLATOR_COUNT;
    Uniforms {
        time,
        freq,
        oscillator_count,
    }
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let storage_dynamic = false;
    let storage_readonly = false;
    let uniform_dynamic = false;
    wgpu::BindGroupLayoutBuilder::new()
        .storage_buffer(
            wgpu::ShaderStages::COMPUTE,
            storage_dynamic,
            storage_readonly,
        )
        .uniform_buffer(wgpu::ShaderStages::COMPUTE, uniform_dynamic)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    oscillator_buffer: &wgpu::Buffer,
    oscillator_buffer_size: wgpu::BufferAddress,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let buffer_size_bytes = std::num::NonZeroU64::new(oscillator_buffer_size).unwrap();
    wgpu::BindGroupBuilder::new()
        .buffer_bytes(oscillator_buffer, 0, Some(buffer_size_bytes))
        .buffer::<Uniforms>(uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nannou"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    })
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    cs_mod: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let desc = wgpu::ComputePipelineDescriptor {
        label: Some("nannou"),
        layout: Some(layout),
        module: &cs_mod,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    };
    device.create_compute_pipeline(&desc)
}

// See `nannou::wgpu::bytes` docs for why these are necessary.

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}
