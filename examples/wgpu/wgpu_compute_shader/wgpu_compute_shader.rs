//! A small GPU compute shader demonstration.
//!
//! Here we use a compute shader to calculate the amplitude of `OSCILLATOR_COUNT` number of
//! oscillators. The oscillator amplitudes are then laid out across the screen using rectangles
//! with a gray value equal to the amplitude. Real-time interaction is demonstrated by providing
//! access to time, frequency (mouse `x`) and the number of oscillators via uniform data.

use nannou::prelude::*;
use std::sync::{Arc, Mutex};

struct Model {
    compute: Compute,
    oscillators: Arc<Mutex<Vec<f32>>>,
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
    let w_id = app.new_window().size(1440, 512).view(view).build().unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();

    // Create the compute shader module.
    let cs = include_bytes!("shaders/comp.spv");
    let cs_spirv = wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap();
    let cs_mod = device.create_shader_module(&cs_spirv);

    // Create the buffer that will store the result of our compute operation.
    let oscillator_buffer_size =
        (OSCILLATOR_COUNT as usize * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
    let oscillator_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: oscillator_buffer_size,
        usage: wgpu::BufferUsage::STORAGE
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    // Create the buffer that will store time.
    let uniforms = create_uniforms(app.time, app.mouse.x, window.rect());
    let uniform_buffer = device
        .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
        .fill_from_slice(&[uniforms]);

    // Create the bind group and pipeline.
    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(
        device,
        &bind_group_layout,
        &oscillator_buffer,
        oscillator_buffer_size,
        &uniform_buffer,
    );
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let pipeline = create_compute_pipeline(device, &pipeline_layout, &cs_mod);

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
        oscillators,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.swap_chain_device();
    let win_rect = window.rect();
    let compute = &mut model.compute;

    // The buffer into which we'll read some data.
    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: compute.oscillator_buffer_size,
        usage: wgpu::BufferUsage::MAP_READ
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    // An update for the uniform buffer with the current time.
    let uniforms = create_uniforms(app.time, app.mouse.x, win_rect);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let new_uniform_buffer = device
        .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(&[uniforms]);

    // The encoder we'll use to encode the compute pass.
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(
        &new_uniform_buffer,
        0,
        &compute.uniform_buffer,
        0,
        uniforms_size,
    );
    {
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&compute.pipeline);
        cpass.set_bind_group(0, &compute.bind_group, &[]);
        cpass.dispatch(OSCILLATOR_COUNT as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        &compute.oscillator_buffer,
        0,
        &read_buffer,
        0,
        compute.oscillator_buffer_size,
    );

    // Submit the compute pass to the device's queue.
    window
        .swap_chain_queue()
        .lock()
        .unwrap()
        .submit(&[encoder.finish()]);

    // Register a callback for reading the result of the compute pass.
    let oscillators = model.oscillators.clone();
    read_buffer.map_read_async(
        0,
        compute.oscillator_buffer_size,
        move |result: wgpu::BufferMapAsyncResult<&[f32]>| {
            if let Ok(mapping) = result {
                if let Ok(mut oscillators) = oscillators.lock() {
                    oscillators.copy_from_slice(&mapping.data);
                }
            }
        },
    );

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

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let draw = app.draw();
    let window = app.window(frame.window_id()).unwrap();
    let rect = window.rect();

    if let Ok(oscillators) = model.oscillators.lock() {
        let w = rect.w() / OSCILLATOR_COUNT as f32;
        let h = rect.h();
        let half_w = w * 0.5;
        for (i, &osc) in oscillators.iter().enumerate() {
            let x = half_w + map_range(i as u32, 0, OSCILLATOR_COUNT, rect.left(), rect.right());
            draw.rect().w_h(w, h).x(x).color(gray(osc));
        }
    }

    draw.to_frame(app, &frame).unwrap();
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
            wgpu::ShaderStage::COMPUTE,
            storage_dynamic,
            storage_readonly,
        )
        .uniform_buffer(wgpu::ShaderStage::COMPUTE, uniform_dynamic)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    oscillator_buffer: &wgpu::Buffer,
    oscillator_buffer_size: wgpu::BufferAddress,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let oscillator_buffer_binding = wgpu::Binding {
        binding: 0,
        resource: wgpu::BindingResource::Buffer {
            buffer: &oscillator_buffer,
            range: 0..oscillator_buffer_size,
        },
    };
    let uniforms_binding = wgpu::Binding {
        binding: 1,
        resource: wgpu::BindingResource::Buffer {
            buffer: &uniform_buffer,
            range: 0..std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        },
    };
    let bindings = &[oscillator_buffer_binding, uniforms_binding];
    device.create_bind_group(&wgpu::BindGroupDescriptor { layout, bindings })
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    })
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    cs_mod: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let compute_stage = wgpu::ProgrammableStageDescriptor {
        module: &cs_mod,
        entry_point: "main",
    };
    let desc = wgpu::ComputePipelineDescriptor {
        layout,
        compute_stage,
    };
    device.create_compute_pipeline(&desc)
}
