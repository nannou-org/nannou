use std::cell::RefCell;
use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use wasm_bindgen::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::util::*;

struct WasmFrame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

thread_local! {
    static PENDING_FRAMES: RefCell<HashMap<u32, WasmFrame>> = RefCell::new(HashMap::new());
    static PENDING_DEVICES: RefCell<Vec<(u32, String)>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn frame_input(device_index: u32, pixel_data: &[u8], width: u32, height: u32) {
    let pixels = pixel_data.to_vec();
    PENDING_FRAMES.with(|cell| {
        cell.borrow_mut().insert(
            device_index,
            WasmFrame {
                pixels,
                width,
                height,
            },
        );
    });
}

#[wasm_bindgen]
pub fn register_device(index: u32, name: &str) {
    PENDING_DEVICES.with(|cell| {
        cell.borrow_mut().push((index, name.to_string()));
    });
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn request_camera(device_index: u32, width: u32, height: u32, framerate: u32);
}

#[derive(Component)]
pub(crate) struct WasmDeviceIndex(pub u32);

#[derive(Component)]
struct WasmStreamDeviceIndex(u32);

pub(crate) fn init(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (enumerate_wasm_devices, open_webcams_wasm, upload_wasm_frames).chain(),
    );

    app.add_observer(on_webcam_removed_wasm);
}

fn enumerate_wasm_devices(
    mut commands: Commands,
    existing_devices: Query<(Entity, &WasmDeviceIndex), With<WebcamDevice>>,
) {
    let pending = PENDING_DEVICES.with(|cell| {
        let mut v = cell.borrow_mut();
        std::mem::take(&mut *v)
    });

    for (index, name) in pending {
        let already_exists = existing_devices.iter().any(|(_, idx)| idx.0 == index);
        if !already_exists {
            let entity = commands
                .spawn((
                    WebcamDevice {
                        name: name.clone(),
                        description: format!("WASM device {index}"),
                        formats: Vec::new(),
                    },
                    WasmDeviceIndex(index),
                ))
                .id();
            commands
                .entity(entity)
                .trigger(|e| WebcamDeviceAdded { entity: e });
        }
    }
}

fn open_webcams_wasm(
    mut commands: Commands,
    new_webcams: Query<(Entity, &Webcam), Added<Webcam>>,
    devices: Query<(Entity, &WasmDeviceIndex), With<WebcamDevice>>,
    active_streams: Query<&WasmStreamDeviceIndex>,
    mut images: ResMut<Assets<Image>>,
) {
    for (stream_entity, webcam) in &new_webcams {
        let device_index = if let Some(device) = webcam.device {
            match devices.get(device) {
                Ok((_, idx)) => idx.0,
                Err(_) => {
                    let msg = "device entity not found";
                    commands
                        .entity(stream_entity)
                        .insert(WebcamError {
                            message: msg.to_string(),
                        });
                    commands.entity(stream_entity).trigger(|e| {
                        WebcamDisconnected {
                            entity: e,
                            reason: msg.to_string(),
                        }
                    });
                    continue;
                }
            }
        } else {
            let in_use: Vec<u32> = active_streams.iter().map(|s| s.0).collect();
            match devices.iter().find(|(_, idx)| !in_use.contains(&idx.0)) {
                Some((_, idx)) => idx.0,
                None => 0, // fallback to device 0 before discovery completes
            }
        };

        let format = frame_texture_format(webcam.srgb);
        let extent = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let image_handle = images.add(Image::new_fill(
            extent,
            TextureDimension::D2,
            &[0u8; 4],
            format,
            RenderAssetUsages::default(),
        ));

        commands.entity(stream_entity).insert((
            WasmStreamDeviceIndex(device_index),
            WebcamStream {
                image: image_handle,
                resolution: UVec2::new(1, 1),
                framerate: 0,
            },
        ));

        let (width, height, fps) = match &webcam.format {
            WebcamFormat::HighestFrameRate => (0, 0, 0),
            WebcamFormat::HighestResolution => (0, 0, 0),
            WebcamFormat::Resolution(res) => (res.x, res.y, 0),
            WebcamFormat::FrameRate(fps) => (0, 0, *fps),
            WebcamFormat::Exact {
                resolution,
                framerate,
            } => (resolution.x, resolution.y, *framerate),
        };
        request_camera(device_index, width, height, fps);

        let resolution = UVec2::new(1, 1);
        commands
            .entity(stream_entity)
            .trigger(|e| WebcamConnected {
                entity: e,
                resolution,
                framerate: 0,
            });
    }
}

fn upload_wasm_frames(
    mut streams: Query<(&WasmStreamDeviceIndex, &Webcam, &mut WebcamStream)>,
    mut images: ResMut<Assets<Image>>,
) {
    let pending: HashMap<u32, WasmFrame> = PENDING_FRAMES.with(|cell| {
        let mut map = cell.borrow_mut();
        std::mem::take(&mut *map)
    });

    for (device_idx, webcam, mut stream) in &mut streams {
        let Some(frame) = pending.get(&device_idx.0) else {
            continue;
        };

        let Some(image) = images.get_mut(&stream.image) else {
            warn!("webcam texture handle is missing");
            continue;
        };

        let extent = Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        };

        let new_resolution = UVec2::new(frame.width, frame.height);
        if stream.resolution != new_resolution {
            stream.resolution = new_resolution;
        }

        let format = frame_texture_format(webcam.srgb);
        write_frame_to_image(image, extent, frame.pixels.clone(), format);
    }
}

fn on_webcam_removed_wasm(event: On<Remove, Webcam>, mut commands: Commands) {
    let entity = event.event_target();
    commands
        .entity(entity)
        .remove::<(WasmStreamDeviceIndex, WebcamStream, WebcamError)>();
}
