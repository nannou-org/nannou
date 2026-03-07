use std::thread;
use std::time::{Duration, Instant};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use flume::{Receiver, Sender};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
    Resolution,
};
use nokhwa::Camera;

use crate::components::*;
use crate::events::*;
use crate::util::*;

pub(crate) struct FramePayload {
    pixels: Vec<u8>,
    extent: Extent3d,
}

struct CaptureWorker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for CaptureWorker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take()
            && let Err(err) = handle.join()
        {
            warn!("capture worker exited with error: {:?}", err);
        }
    }
}

#[derive(Component)]
pub(crate) struct WebcamCapture {
    receiver: Receiver<FramePayload>,
    _worker: CaptureWorker,
    is_srgb: bool,
    device_entity: Entity,
}

#[derive(Component)]
pub(crate) struct NativeDeviceIndex(pub CameraIndex);

#[derive(Resource)]
struct DeviceEnumerationTimer {
    last_check: Instant,
}

pub(crate) fn init(app: &mut App) {
    #[cfg(target_os = "macos")]
    nokhwa_initialize_blocking();

    app.insert_resource(DeviceEnumerationTimer {
        last_check: Instant::now() - Duration::from_secs(10),
    });

    app.add_systems(
        PreUpdate,
        (enumerate_webcam_devices, open_webcams, upload_native_frames).chain(),
    );

    app.add_observer(on_webcam_removed);
}

#[cfg(target_os = "macos")]
fn nokhwa_initialize_blocking() {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    nokhwa::nokhwa_initialize(move |success| {
        let _ = tx.send(success);
    });

    match rx.recv() {
        Ok(true) => {}
        Ok(false) => panic!("user denied camera permission"),
        Err(_) => panic!("initialization channel closed unexpectedly"),
    }
}

fn query_device_formats(index: &CameraIndex) -> Vec<WebcamSupportedFormat> {
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let mut camera = match Camera::new(index.clone(), requested) {
        Ok(c) => c,
        Err(err) => {
            warn!("failed to query device formats: {err}");
            return Vec::new();
        }
    };

    match camera.compatible_camera_formats() {
        Ok(formats) => formats
            .into_iter()
            .map(|f| WebcamSupportedFormat {
                resolution: UVec2::new(f.resolution().width_x, f.resolution().height_y),
                framerate: f.frame_rate(),
            })
            .collect(),
        Err(err) => {
            warn!("failed to query compatible formats: {err}");
            Vec::new()
        }
    }
}

fn enumerate_webcam_devices(
    mut commands: Commands,
    existing_devices: Query<(Entity, &NativeDeviceIndex), With<WebcamDevice>>,
    active_captures: Query<(Entity, &WebcamCapture)>,
    mut timer: ResMut<DeviceEnumerationTimer>,
) {
    let now = Instant::now();
    if now.duration_since(timer.last_check) < Duration::from_secs(2) {
        return;
    }
    timer.last_check = now;

    let discovered = match nokhwa::query(ApiBackend::Auto) {
        Ok(devices) => devices,
        Err(err) => {
            warn!("failed to enumerate webcam devices: {err}");
            return;
        }
    };

    let discovered_indices: Vec<_> = discovered.iter().map(|d| d.index().clone()).collect();

    for info in &discovered {
        let index = info.index().clone();
        let already_exists = existing_devices.iter().any(|(_, idx)| idx.0 == index);
        if !already_exists {
            let formats = query_device_formats(&index);
            let entity = commands
                .spawn((
                    WebcamDevice {
                        name: info.human_name().to_string(),
                        description: info.description().to_string(),
                        formats,
                    },
                    NativeDeviceIndex(index),
                ))
                .id();
            commands
                .entity(entity)
                .trigger(|e| WebcamDeviceAdded { entity: e });
        }
    }

    for (entity, idx) in &existing_devices {
        if !discovered_indices.contains(&idx.0) {
            for (stream_entity, capture) in &active_captures {
                if capture.device_entity == entity {
                    commands
                        .entity(stream_entity)
                        .remove::<(WebcamCapture, WebcamStream)>();
                    commands.entity(stream_entity).trigger(|e| {
                        WebcamDisconnected {
                            entity: e,
                            reason: "device removed".to_string(),
                        }
                    });
                }
            }
            commands
                .entity(entity)
                .trigger(|e| WebcamDeviceRemoved { entity: e });
            commands.entity(entity).despawn();
        }
    }
}

fn open_webcams(
    mut commands: Commands,
    new_webcams: Query<(Entity, &Webcam), Added<Webcam>>,
    devices: Query<(Entity, &NativeDeviceIndex), With<WebcamDevice>>,
    active_captures: Query<&WebcamCapture>,
    mut images: ResMut<Assets<Image>>,
) {
    for (stream_entity, webcam) in &new_webcams {
        let device_entity = if let Some(device) = webcam.device {
            device
        } else {
            let in_use: Vec<Entity> = active_captures.iter().map(|c| c.device_entity).collect();
            match devices.iter().find(|(e, _)| !in_use.contains(e)) {
                Some((e, _)) => e,
                None => {
                    let msg = "no available webcam device found";
                    warn!("{msg}");
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
        };

        if active_captures
            .iter()
            .any(|c| c.device_entity == device_entity)
        {
            let msg = "device already in use by another stream";
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

        let Ok((_, native_index)) = devices.get(device_entity) else {
            let msg = "referenced device entity not found";
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
        };

        let format_type = match webcam.format {
            WebcamFormat::HighestFrameRate => RequestedFormatType::AbsoluteHighestFrameRate,
            WebcamFormat::HighestResolution => RequestedFormatType::AbsoluteHighestResolution,
            WebcamFormat::Resolution(res) => {
                RequestedFormatType::HighestResolution(Resolution::new(res.x, res.y))
            }
            WebcamFormat::FrameRate(fps) => RequestedFormatType::HighestFrameRate(fps),
            WebcamFormat::Exact {
                resolution,
                framerate,
            } => RequestedFormatType::Closest(CameraFormat::new(
                Resolution::new(resolution.x, resolution.y),
                FrameFormat::MJPEG,
                framerate,
            )),
        };

        let requested = RequestedFormat::new::<RgbFormat>(format_type);
        let camera = match Camera::new(native_index.0.clone(), requested) {
            Ok(c) => c,
            Err(err) => {
                let msg = format!("failed to create camera: {err}");
                commands
                    .entity(stream_entity)
                    .insert(WebcamError {
                        message: msg.clone(),
                    });
                let reason = msg;
                commands
                    .entity(stream_entity)
                    .trigger(|e| WebcamDisconnected { entity: e, reason });
                continue;
            }
        };

        match open_camera_stream(camera, webcam.srgb, device_entity, &mut images) {
            Ok((capture, stream)) => {
                let resolution = stream.resolution;
                let framerate = stream.framerate;
                commands.entity(stream_entity).insert((capture, stream));
                commands
                    .entity(stream_entity)
                    .trigger(|e| WebcamConnected {
                        entity: e,
                        resolution,
                        framerate,
                    });
            }
            Err(msg) => {
                commands
                    .entity(stream_entity)
                    .insert(WebcamError {
                        message: msg.clone(),
                    });
                let reason = msg;
                commands
                    .entity(stream_entity)
                    .trigger(|e| WebcamDisconnected { entity: e, reason });
            }
        }
    }
}

fn open_camera_stream(
    mut camera: Camera,
    is_srgb: bool,
    device_entity: Entity,
    images: &mut Assets<Image>,
) -> Result<(WebcamCapture, WebcamStream), String> {
    camera
        .open_stream()
        .map_err(|e| format!("failed to open camera stream: {e}"))?;

    let framerate = camera.frame_rate();
    let resolution = camera.resolution();
    info!(
        "camera framerate: {framerate}, resolution: {}x{}",
        resolution.width_x, resolution.height_y
    );

    let extent = Extent3d {
        width: resolution.width_x,
        height: resolution.height_y,
        depth_or_array_layers: 1,
    };

    let format = frame_texture_format(is_srgb);
    let image_handle = images.add(Image::new_fill(
        extent,
        TextureDimension::D2,
        &[0u8; 4],
        format,
        RenderAssetUsages::default(),
    ));

    let (sender, receiver) = flume::bounded(2);
    let handle = thread::Builder::new()
        .name("bevy_webcam_capture".to_string())
        .spawn(move || capture_frames(camera, sender))
        .map_err(|e| format!("failed to spawn capture thread: {e}"))?;

    Ok((
        WebcamCapture {
            receiver,
            _worker: CaptureWorker {
                handle: Some(handle),
            },
            is_srgb,
            device_entity,
        },
        WebcamStream {
            image: image_handle,
            resolution: UVec2::new(resolution.width_x, resolution.height_y),
            framerate,
        },
    ))
}

fn upload_native_frames(
    mut captures: Query<(&WebcamCapture, &mut WebcamStream)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (capture, mut stream) in &mut captures {
        let mut latest_frame = None;
        while let Ok(frame) = capture.receiver.try_recv() {
            latest_frame = Some(frame);
        }

        let Some(frame) = latest_frame else {
            continue;
        };

        let Some(mut image) = images.get_mut(&stream.image) else {
            warn!("webcam texture handle is missing");
            continue;
        };

        let format = frame_texture_format(capture.is_srgb);
        let new_resolution = UVec2::new(frame.extent.width, frame.extent.height);
        if stream.resolution != new_resolution {
            warn!(
                "camera resolution changed from {}x{} to {}x{}",
                stream.resolution.x,
                stream.resolution.y,
                new_resolution.x,
                new_resolution.y,
            );
            stream.resolution = new_resolution;
        }

        write_frame_to_image(&mut *image, frame.extent, frame.pixels, format);
    }
}

fn capture_frames(mut camera: Camera, sender: Sender<FramePayload>) {
    loop {
        match camera.frame() {
            Ok(frame) => match frame.decode_image::<RgbFormat>() {
                Ok(image) => {
                    let (width, height) = image.dimensions();
                    let extent = Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    };
                    let rgba_pixels = rgb_to_rgba(&image.into_raw());
                    if sender
                        .send(FramePayload {
                            pixels: rgba_pixels,
                            extent,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(err) => error!("failed to decode camera frame: {err}"),
            },
            Err(err) => {
                error!("failed to get camera frame: {err}");
                thread::sleep(Duration::from_millis(16));
            }
        }
    }
}

fn on_webcam_removed(event: On<Remove, Webcam>, mut commands: Commands) {
    let entity = event.event_target();
    commands
        .entity(entity)
        .remove::<(WebcamCapture, WebcamStream, WebcamError)>();
}
