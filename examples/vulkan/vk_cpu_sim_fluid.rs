//! A crude port of Daniel Shiffman's Fluid Simulation [Coding Challenge](https://www.youtube.com/watch?v=alhpH6ECFvQ)
//!
//! Actual simulation code is ported from [Mike Ash's thesis](https://mikeash.com/pyblog/fluid-simulation-for-dummies.html) by D.Shiffman
//! 
//! The example actually makes use of Vulkan for drawing the density data. In order to do that we introduce a texture, dynamically
//! created by the CPU and fed into as a uniform.
//! 
//! This also uses Perlin noise to create velocity vectors.
//! 
//! 
//! 

use nannou::prelude::*;
use nannou::noise::{NoiseFn, Perlin};
use nannou::math::Rad;
use std::cell::RefCell;
use std::sync::Arc;

const GRID_SIZE: u32 = 128;
const ITER_NUM: usize = 4;
const SCALE: u32 = 4;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    timestep: f32,
    diffusion: f32,
    viscosity: f32,

    density0: Vec<f32>,
    density: Vec<f32>,

    vx0: Vec<f32>,
    vy0: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,

    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
    sampler: Arc<vk::Sampler>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    view_fbo: RefCell<ViewFbo>,

    perlin: Perlin,
    time: f64,
}

#[derive(Debug, Default, Clone)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

impl Model {
    fn add_density(&mut self, x: usize, y: usize, amount: f32) {
        self.density[ix(x, y)] += amount;
    }

    fn add_velocity(&mut self, x: usize, y: usize, amount_x: f32, amount_y: f32) {
        let idx = ix(x, y);

        self.vx[idx] += amount_x;
        self.vy[idx] += amount_y;
    }
}

fn model(app: &App) -> Model {
    let facade = SCALE * GRID_SIZE;
    let gsize = GRID_SIZE as usize;

    app.new_window()
        .with_dimensions(facade, facade)
        .view(view)
        .build()
        .unwrap();

    let device = app.main_window().swapchain().device().clone();

    let vertex_buffer = {
        // Cover the whole viewport space with two triangles.
        let positions = [
            [-1.0, -1.0],
            [-1.0,  1.0],
            [ 1.0, -1.0],
            [ 1.0,  1.0],
        ];
        let vertices = positions.iter().map(|&position| Vertex {position});

        vk::CpuAccessibleBuffer::from_iter(device.clone(), vk::BufferUsage::all(), vertices).unwrap()
    };

    let vertex_shader = vs::Shader::load(device.clone()).unwrap();
    let fragment_shader = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vk::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: nannou::frame::COLOR_FORMAT,
                    samples: app.main_window().msaa_samples(),
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    let pipeline = Arc::new(
        vk::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_strip()  // This makes 2 triagles with 4 vertices.
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .render_pass(vk::Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let view_fbo = RefCell::new(ViewFbo::default());

    let sampler = vk::SamplerBuilder::new().build(device.clone()).unwrap();

    let perlin = Perlin::new();

    Model {
        timestep: 0.01,  // Used for driving Perlin noise
        diffusion: 0.0,
        viscosity: 0.0000001,
        density0: vec![0.0; gsize * gsize],
        density: vec![0.0; gsize * gsize],
        vx0: vec![0.0; gsize * gsize],
        vy0: vec![0.0; gsize * gsize],
        vx: vec![0.0; gsize * gsize],
        vy: vec![0.0; gsize * gsize],
        render_pass,
        pipeline,
        sampler,
        vertex_buffer,
        view_fbo,
        perlin,
        time: 0.0,
    }
}

fn get_point(mouse_x: f32, mouse_y: f32) -> (usize, usize) {
    let shift = (SCALE * GRID_SIZE)  as f32 / 2.0;
    let mx = (mouse_x + shift) / SCALE as f32;
    let my = (mouse_y + shift) / SCALE as f32;

    (mx.floor() as usize, my.floor() as usize)
}

fn view(app: &App, m: &Model, frame: &Frame) {
    let [w, h] = frame.swapchain_image().dimensions();
    let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
    let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);

    m.view_fbo.borrow_mut().update(frame, m.render_pass.clone(), |builder, image| {
        builder.add(image)
    }).unwrap();

    let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into()];

    // Create a density image on the fly by going through density values.
    let (density_texture, _tex) = {
        vk::ImmutableImage::from_iter(
            m.density.iter().map(|x| {
                let xx = (x.clone() * 256.0) as u8;
                // Clever color manipulations are possible, but we go with red hue
                (xx, 0, 0, 255)
            }),
            vk::image::Dimensions::Dim2d {
                width: GRID_SIZE,
                height: GRID_SIZE
            },
            vk::format::Format::R8G8B8A8Srgb,
            app.main_window().swapchain_queue().clone(),
        ).unwrap()
    };

    let density_data_set = Arc::new(
        vk::PersistentDescriptorSet::start(m.pipeline.clone(), 0)
        .add_sampled_image(density_texture.clone(), m.sampler.clone())
        .unwrap()
        .build()
        .unwrap()
    );

    frame
        .add_commands()
        .begin_render_pass(m.view_fbo.borrow().expect_inner(), clear_values)
        .unwrap()
        .draw(
            m.pipeline.clone(),
            &dynamic_state,
            vec![m.vertex_buffer.clone()],
            density_data_set,
            (),
        )
        .unwrap()
        .end_render_pass()
        .expect("failed to add `end_render_pass` command");
}

fn update(app: &App, mut m: &mut Model, _update: Update) {
    let p = get_point(app.mouse.x, app.mouse.y);
    m.add_density(p.0, p.1, 1.0);
    let ang = m.perlin.get([m.time, 42.0 * m.time]);
    let r = Rad(ang * 360.0 * 2.0);

    let c = GRID_SIZE as usize / 2;
    let s = GRID_SIZE as f32 * 3.0;

    m.add_velocity(c, c, s * r.sin() as f32, s * r.cos() as f32);

    step(&mut m);
    m.time += 0.01;
}

mod vs {
    nannou::vk::shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    
    // These coordinates are used for sampling image, so it's coordinate
    // system goes from (0, 0) to positive number pairs. Thus we map
    // Vulkan's coordinate system to the image's coordinate system.
    // Keeping in mind that Vulkan viewport has y axis flipped with 
    // respect to nannou.
    tex_coords = (position * vec2(1.0, -1.0) + vec2(1.0)) / vec2(2.0);
}
"
    }
}

mod fs {
    nannou::vk::shaders::shader! {
    ty: "fragment",
    src: "
#version 450

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = texture(tex, tex_coords);
}
"
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////
// Actual port of the fluid sim code.
///////////////////////////////////////////////////////////////////////////////////////////////
fn ix(i: usize, j: usize) -> usize {
    let gs = GRID_SIZE as usize;
    let x = if i >= gs {gs - 1} else {i};
    let y = if j >= gs {gs - 1} else {j};
    let k = x + y * gs;
    k
}

fn set_bnd(b: usize, x: &mut Vec<f32>) {
    // This is basically reflective boundary condition.
    let n = GRID_SIZE as usize;

    for i in 1..n-1 {
        x[ix(i, 0  )] = if b == 2 {-x[ix(i, 1  )]} else {x[ix(i, 1  )]};
        x[ix(i, n-1)] = if b == 2 {-x[ix(i, n-2)]} else {x[ix(i, n-2)]};
    }

    for j in 1..n-1 {
        x[ix(0  , j)] = if b == 1 {-x[ix(1  , j)]} else {x[ix(1  , j)]};
        x[ix(n-1, j)] = if b == 1 {-x[ix(n-2, j)]} else {x[ix(n-2, j)]};
    }

    x[ix(0, 0)] = 0.5 * (x[ix(1, 0)] + x[ix(0, 1)]);
    x[ix(0, n-1)] = 0.5 * (x[ix(1, n-1)] + x[ix(0, n-2)]);
    x[ix(n-1, 0)] = 0.5 * (x[ix(n-2, 0)] + x[ix(n-1, 1)]);
    x[ix(n-1, n-1)] = 0.5 * (x[ix(n-2, n-1)] + x[ix(n-1, n-2)]);
}

fn lin_solve(b: usize, x: &mut Vec<f32>, x0: &Vec<f32>, alpha: f32, beta: f32) {
    // He is using Jacobi method to solve the linear differential equation
    let n = GRID_SIZE as usize;
    let beta_recip = 1.0 / beta;
    for _k in 0..ITER_NUM {
        for j in 1..n-1 {
            for i in 1..n-1 {
                x[ix(i, j)] =(x0[ix(i, j)]
                    + alpha*(x[ix(i+1, j  )]
                            +x[ix(i-1, j  )]
                            +x[ix(i  , j+1)]
                            +x[ix(i  , j-1)]
                            +x[ix(i  , j  )]
                            +x[ix(i  , j  )]
                    )) * beta_recip;
            }
        }
        set_bnd(b, x);
    }
}

fn advect(b: usize, d: &mut Vec<f32>, d0: &Vec<f32>,  vx: &Vec<f32>, vy: &Vec<f32>, dt:f32) {
    let n = GRID_SIZE as usize;
    let nfloat = n as f32;

    let mut i0: f32;
    let mut i1: f32;
    let mut j0: f32;
    let mut j1: f32;

    let dtx = dt * (n as f32 - 2.0);
    let dty = dt * (n as f32 - 2.0);

    let mut s0: f32;
    let mut s1: f32;
    let mut t0: f32;
    let mut t1: f32;

    let mut tmp1: f32;
    let mut tmp2: f32;
    let mut x: f32;
    let mut y: f32;

    for j in 1..n-1 {
        for i in 1..n-1 {
            tmp1 = dtx * vx[ix(i, j)];
            tmp2 = dty * vy[ix(i, j)];

            // It looks like we are going back in time
            // However this method is actually useful
            // for making the whole calculation independent
            // of state, and making the calculation stable.
            x = i as f32 - tmp1;
            y = j as f32 - tmp2;

            // All the faff below is for interpolation.
            if x < 0.5 {x = 0.5;}
            if x > nfloat + 0.5 {x = nfloat + 0.5;}
            i0 = x.floor();
            i1 = i0 + 1.0;

            if y < 0.5 {y = 0.5;}
            if y > nfloat + 0.5 {y = nfloat + 0.5;}
            j0 = y.floor();
            j1 = j0 + 1.0;

            s1 = x - i0;
            s0 = 1.0 - s1;
            t1 = y - j0;
            t0 = 1.0 - t1;

            let i0i = i0 as usize;
            let i1i = i1 as usize;
            let j0i = j0 as usize;
            let j1i = j1 as usize;

            d[ix(i, j)] =
                s0 * ( t0 * d0[ix(i0i, j0i)] + t1 * d0[ix(i0i, j1i)])
               +s1 * ( t0 * d0[ix(i1i, j0i)] + t1 * d0[ix(i1i, j1i)]);
        }
    }
    set_bnd(b, d);
}

fn diffuse(b: usize, x: &mut Vec<f32>, x0: &Vec<f32>, diff: f32, dt: f32) {
    let n = GRID_SIZE as f32;
    let alpha = dt * diff * (n-2.0) * (n-2.0);
    let beta = 1.0 + 6.0 * alpha;
    lin_solve(b, x, x0, alpha, beta);
}


fn project(vx: &mut Vec<f32>, vy: &mut Vec<f32>, p: &mut Vec<f32>, div: &mut Vec<f32>) {
    let n = GRID_SIZE as usize;
    let nfloat = n as f32;

    for j in 1..n-1 {
        for i in 1..n-1 {
            div[ix(i, j)] = -0.5*(
                     vx[ix(i+1, j  )]
                    -vx[ix(i-1, j  )]
                    +vy[ix(i  , j+1)]
                    -vy[ix(i  , j-1)]
                ) / nfloat;
            p[ix(i, j)] = 0.0;
        }
    }

    set_bnd(0, div);
    set_bnd(0, p);
    lin_solve(0, p, div, 1.0, 6.0);

    for j in 1..n-1 {
        for i in 1..n-1 {
            vx[ix(i, j)] -= 0.5 * (p[ix(i+1, j)]
                                  -p[ix(i-1, j)]) * nfloat;
            vy[ix(i, j)] -= 0.5 * (p[ix(i, j+1)]
                                  -p[ix(i, j-1)]) * nfloat;
        }
    }

    set_bnd(1, vx);
    set_bnd(2, vy);
}

fn step(cube: &mut Model) {
    // int N          = cube->size;
    let visc        = cube.viscosity;
    let diff        = cube.diffusion;
    let dt          = cube.timestep;
    let mut vx      = &mut cube.vx;
    let mut vy      = &mut cube.vy;
    let mut vx0     = &mut cube.vx0;
    let mut vy0     = &mut cube.vy0;
    let mut density = &mut cube.density;
    let mut s       = &mut cube.density0;

    diffuse(1, &mut vx0, &vx, visc, dt);
    diffuse(2, &mut vy0, &vy, visc, dt);

    project(&mut vx0, &mut vy0, &mut vx, &mut vy);

    advect(1, &mut vx, &vx0, &vx0, &vy0, dt);
    advect(2, &mut vy, &vy0, &vx0, &vy0, dt);

    project(&mut vx, &mut vy, &mut vx0, &mut vy0);

    diffuse(0, &mut s, &density, diff, dt);
    advect(0, &mut density, &s, &vx, &vy, dt);
}
