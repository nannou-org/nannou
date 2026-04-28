use std::collections::HashMap;
use std::sync::Arc;

use bevy::app::SubApp;
use bevy::asset::io::Reader;
use bevy::asset::{AssetId, AssetLoader, LoadContext, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::{
    Extract, ExtractSchedule, Render, RenderSystems,
    render_asset::RenderAssets,
    render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    renderer::RenderQueue,
    texture::GpuImage,
};
use flume::{Receiver, Sender};
use gloo_events::EventListener;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, Document, HtmlVideoElement, Url};
use wgpu_types::{
    CopyExternalImageDestInfo, CopyExternalImageSourceInfo, ExternalImageSource, Origin2d,
    Origin3d, PredefinedColorSpace, TextureAspect,
};

use crate::asset::{Video, VideoAssetLoaderError, VideoLoaderSettings, VideoSource};
use crate::components::{PlaybackMode, SeekTo, VideoOutput, VideoPlayer};
use crate::events::{VideoEnded, VideoFailed, VideoLoaded, VideoSeeked};

#[derive(Default, TypePath)]
pub(crate) struct VideoLoader;

impl AssetLoader for VideoLoader {
    type Asset = Video;
    type Settings = VideoLoaderSettings;
    type Error = VideoAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Video::probe(VideoSource::Bytes(Arc::new(bytes)), settings)
    }

    fn extensions(&self) -> &[&str] {
        &["mp4", "mov", "mkv", "webm", "avi", "m4v", "ts"]
    }
}

#[derive(Clone, Copy, Debug)]
enum DomEvent {
    LoadedMetadata,
    Playing,
    Ended,
    Error,
}

pub(crate) struct VideoRegistry {
    elements: HashMap<Entity, ManagedElement>,
    document: Document,
    tx: Sender<(Entity, DomEvent)>,
    rx: Receiver<(Entity, DomEvent)>,
}

struct ManagedElement {
    element: HtmlVideoElement,
    image_id: AssetId<Image>,
    blob_url: Option<String>,
    renderable: bool,
    last_reported_size: UVec2,
    _listeners: Vec<EventListener>,
}

impl Drop for ManagedElement {
    fn drop(&mut self) {
        let _ = self.element.pause();
        self.element.set_src("");
        if let Some(ref url) = self.blob_url {
            let _ = Url::revoke_object_url(url);
        }
    }
}

impl VideoRegistry {
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let (tx, rx) = flume::unbounded();
        Ok(Self {
            elements: HashMap::new(),
            document,
            tx,
            rx,
        })
    }
}

#[derive(Component)]
pub(crate) struct PendingVideo;

#[derive(Component)]
pub(crate) struct VideoAttached;

pub(crate) fn install_registry(app: &mut App) {
    match VideoRegistry::new() {
        Ok(reg) => {
            app.insert_non_send(reg);
        }
        Err(err) => {
            error!("nannou_video: failed to initialize registry: {err:?}");
        }
    }
}

pub(crate) fn attach_players(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    videos: Res<Assets<Video>>,
    added: Query<(Entity, &VideoPlayer), (Added<VideoPlayer>, Without<VideoAttached>)>,
    pending: Query<(Entity, &VideoPlayer), (With<PendingVideo>, Without<VideoAttached>)>,
    mut registry: NonSendMut<VideoRegistry>,
) {
    for (entity, player) in added.iter().chain(pending.iter()) {
        let Some(video) = videos.get(&player.video) else {
            commands.entity(entity).insert(PendingVideo);
            continue;
        };

        let (element, blob_url) = match create_element(&registry.document, &video.source) {
            Ok(v) => v,
            Err(err) => {
                commands.entity(entity).trigger(|e| VideoFailed {
                    entity: e,
                    reason: format!("{err:?}"),
                });
                continue;
            }
        };

        element.set_muted(true);
        element.set_loop(matches!(player.mode, PlaybackMode::Loop));
        element.set_playback_rate(player.speed as f64);
        element.set_cross_origin(Some("anonymous"));
        if !player.paused {
            let _ = element.play();
        }

        let mut image = Image::new_fill(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
        let image_handle = images.add(image);
        let image_id = image_handle.id();

        let listeners = attach_listeners(&element, entity, &registry.tx);

        registry.elements.insert(
            entity,
            ManagedElement {
                element,
                image_id,
                blob_url,
                renderable: false,
                last_reported_size: UVec2::ZERO,
                _listeners: listeners,
            },
        );

        commands.entity(entity).insert((
            VideoOutput {
                image: image_handle,
                size: UVec2::ZERO,
                position_seconds: 0.0,
            },
            VideoAttached,
        ));
        commands.entity(entity).remove::<PendingVideo>();
        commands
            .entity(entity)
            .trigger(|e| VideoLoaded { entity: e });
    }
}

fn create_element(
    document: &Document,
    source: &VideoSource,
) -> Result<(HtmlVideoElement, Option<String>), JsValue> {
    let element: HtmlVideoElement = document.create_element("video")?.dyn_into()?;
    element.set_autoplay(true);
    element.set_attribute("playsinline", "true")?;
    let blob_url = match source {
        VideoSource::Url(url) => {
            element.set_src(url);
            None
        }
        VideoSource::Bytes(bytes) => {
            let array = Uint8Array::new_with_length(bytes.len() as u32);
            array.copy_from(bytes);
            let parts = Array::new();
            parts.push(&array);
            let opts = BlobPropertyBag::new();
            opts.set_type("video/mp4");
            let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)?;
            let url = Url::create_object_url_with_blob(&blob)?;
            element.set_src(&url);
            Some(url)
        }
    };
    Ok((element, blob_url))
}

fn attach_listeners(
    element: &HtmlVideoElement,
    entity: Entity,
    tx: &Sender<(Entity, DomEvent)>,
) -> Vec<EventListener> {
    let events = [
        ("loadedmetadata", DomEvent::LoadedMetadata),
        ("playing", DomEvent::Playing),
        ("ended", DomEvent::Ended),
        ("error", DomEvent::Error),
    ];
    events
        .into_iter()
        .map(|(name, kind)| {
            let tx = tx.clone();
            EventListener::new(element, name, move |_| {
                let _ = tx.send((entity, kind));
            })
        })
        .collect()
}

pub(crate) fn sync_commands(
    players: Query<(Entity, &VideoPlayer), Changed<VideoPlayer>>,
    registry: NonSend<VideoRegistry>,
) {
    for (entity, player) in &players {
        let Some(managed) = registry.elements.get(&entity) else {
            continue;
        };
        managed
            .element
            .set_loop(matches!(player.mode, PlaybackMode::Loop));
        managed.element.set_playback_rate(player.speed as f64);
        if player.paused {
            let _ = managed.element.pause();
        } else {
            let _ = managed.element.play();
        }
    }
}

pub(crate) fn process_seeks(
    mut commands: Commands,
    seeks: Query<(Entity, &SeekTo)>,
    registry: NonSend<VideoRegistry>,
) {
    for (entity, seek) in &seeks {
        if let Some(managed) = registry.elements.get(&entity) {
            managed.element.set_current_time(seek.0);
            commands.entity(entity).trigger(|e| VideoSeeked {
                entity: e,
                to_seconds: seek.0,
            });
        }
        commands.entity(entity).remove::<SeekTo>();
    }
}

pub(crate) fn drain_events(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut outputs: Query<&mut VideoOutput>,
    mut videos: ResMut<Assets<Video>>,
    query: Query<&VideoPlayer>,
    mut registry: NonSendMut<VideoRegistry>,
) {
    let rx = registry.rx.clone();
    let events: Vec<_> = rx.try_iter().collect();
    for (entity, kind) in events {
        let Some(managed) = registry.elements.get_mut(&entity) else {
            continue;
        };
        match kind {
            DomEvent::LoadedMetadata => {
                let width = managed.element.video_width();
                let height = managed.element.video_height();
                let size = UVec2::new(width, height);
                if width != 0 && height != 0 && size != managed.last_reported_size {
                    resize_target(&mut images, managed.image_id, size);
                    managed.last_reported_size = size;
                    if let Ok(mut output) = outputs.get_mut(entity) {
                        output.size = size;
                    }
                    if let Ok(player) = query.get(entity)
                        && let Some(mut video) = videos.get_mut(&player.video)
                    {
                        if video.size == UVec2::ZERO {
                            video.size = size;
                        }
                        let duration = managed.element.duration();
                        if duration.is_finite() && duration > 0.0 && video.duration_seconds.is_none()
                        {
                            video.duration_seconds = Some(duration);
                        }
                    }
                }
            }
            DomEvent::Playing => {
                managed.renderable = true;
            }
            DomEvent::Ended => {
                managed.renderable = false;
                commands
                    .entity(entity)
                    .trigger(|e| VideoEnded { entity: e });
            }
            DomEvent::Error => {
                managed.renderable = false;
                let reason = <HtmlVideoElement as AsRef<web_sys::HtmlMediaElement>>::as_ref(
                    &managed.element,
                )
                .error()
                .map(|e| format!("media error code {}: {}", e.code(), e.message()))
                .unwrap_or_else(|| "unknown media error".to_string());
                commands
                    .entity(entity)
                    .trigger(|e| VideoFailed { entity: e, reason });
            }
        }
    }
}

pub(crate) fn sync_positions(
    mut players: Query<(Entity, &mut VideoOutput)>,
    registry: NonSend<VideoRegistry>,
) {
    for (entity, mut output) in &mut players {
        if let Some(managed) = registry.elements.get(&entity) {
            output.position_seconds = managed.element.current_time();
        }
    }
}

pub(crate) fn on_player_removed(
    event: On<Remove, VideoPlayer>,
    mut commands: Commands,
    mut registry: NonSendMut<VideoRegistry>,
) {
    let entity = event.event_target();
    registry.elements.remove(&entity);
    commands
        .entity(entity)
        .remove::<(VideoOutput, VideoAttached, PendingVideo, SeekTo)>();
}

fn resize_target(images: &mut Assets<Image>, id: AssetId<Image>, size: UVec2) {
    let Some(mut image) = images.get_mut(id) else {
        return;
    };
    image.texture_descriptor.size = Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    };
    image.data = None;
}

#[derive(Default)]
pub(crate) struct RenderElements {
    entries: Vec<RenderEntry>,
}

struct RenderEntry {
    element: HtmlVideoElement,
    image_id: AssetId<Image>,
}

pub(crate) fn install_render_app(render_app: &mut SubApp) {
    render_app
        .world_mut()
        .insert_non_send(RenderElements::default());
    render_app
        .add_systems(ExtractSchedule, extract_elements)
        .add_systems(Render, queue_copies.in_set(RenderSystems::PrepareAssets));
}

fn extract_elements(
    registry: Extract<NonSend<VideoRegistry>>,
    mut render: NonSendMut<RenderElements>,
) {
    render.entries.clear();
    for managed in registry.elements.values() {
        if managed.renderable && managed.last_reported_size != UVec2::ZERO {
            render.entries.push(RenderEntry {
                element: managed.element.clone(),
                image_id: managed.image_id,
            });
        }
    }
}

fn queue_copies(
    queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render: NonSend<RenderElements>,
) {
    for entry in &render.entries {
        let Some(gpu_image) = gpu_images.get(entry.image_id) else {
            continue;
        };
        queue.0.copy_external_image_to_texture(
            &CopyExternalImageSourceInfo {
                source: ExternalImageSource::HTMLVideoElement(entry.element.clone()),
                origin: Origin2d::ZERO,
                flip_y: false,
            },
            CopyExternalImageDestInfo {
                texture: &gpu_image.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
                color_space: PredefinedColorSpace::Srgb,
                premultiplied_alpha: true,
            },
            gpu_image.texture_descriptor.size,
        );
    }
}
