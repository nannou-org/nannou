use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::asset::Video;
use crate::components::{SeekTo, VideoOutput, VideoPlayer};
use crate::events::{VideoEnded, VideoFailed, VideoLoaded, VideoLooped, VideoSeeked};
use crate::worker::{FrameEvent, FramePayload, PlayerCommand, VideoWorker, WorkerConfig, spawn_worker};

#[derive(Component)]
pub(crate) struct PendingVideo;

pub(crate) fn attach_workers(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    videos: Res<Assets<Video>>,
    added: Query<(Entity, &VideoPlayer), (Added<VideoPlayer>, Without<VideoWorker>)>,
    pending: Query<(Entity, &VideoPlayer), (With<PendingVideo>, Without<VideoWorker>)>,
) {
    for (entity, player) in added.iter().chain(pending.iter()) {
        let Some(video) = videos.get(&player.video) else {
            commands.entity(entity).insert(PendingVideo);
            continue;
        };
        let image = Image::new_fill(
            Extent3d {
                width: video.size.x,
                height: video.size.y,
                ..default()
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let image_handle = images.add(image);
        let worker = spawn_worker(WorkerConfig {
            source: video.source.clone(),
            mode: player.mode,
            hw_accel: player.hw_accel.resolve(),
            resize: player.resize.resolve(),
            options: video.options.clone(),
        });
        if player.paused {
            let _ = worker.cmd_tx.send(PlayerCommand::Pause);
        }
        if (player.speed - 1.0).abs() > f32::EPSILON {
            let _ = worker.cmd_tx.send(PlayerCommand::SetSpeed(player.speed));
        }
        commands.entity(entity).insert((
            VideoOutput {
                image: image_handle,
                size: video.size,
                position_seconds: 0.0,
            },
            worker,
        ));
        commands.entity(entity).remove::<PendingVideo>();
        commands
            .entity(entity)
            .trigger(|e| VideoLoaded { entity: e });
    }
}

pub(crate) fn process_seeks(
    mut commands: Commands,
    seeks: Query<(Entity, &VideoWorker, &SeekTo)>,
) {
    for (entity, worker, seek) in &seeks {
        let _ = worker.cmd_tx.send(PlayerCommand::Seek(seek.0));
        commands.entity(entity).remove::<SeekTo>();
        commands.entity(entity).trigger(|e| VideoSeeked {
            entity: e,
            to_seconds: seek.0,
        });
    }
}

pub(crate) fn sync_commands(players: Query<(&VideoPlayer, &VideoWorker), Changed<VideoPlayer>>) {
    for (player, worker) in &players {
        let _ = worker.cmd_tx.send(if player.paused {
            PlayerCommand::Pause
        } else {
            PlayerCommand::Play
        });
        let _ = worker.cmd_tx.send(PlayerCommand::SetSpeed(player.speed));
        let _ = worker.cmd_tx.send(PlayerCommand::SetMode(player.mode));
    }
}

pub(crate) fn drain_frames(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut players: Query<(Entity, &VideoWorker, &mut VideoOutput)>,
) {
    for (entity, worker, mut output) in &mut players {
        let mut latest_frame: Option<FramePayload> = None;
        while let Ok(event) = worker.frame_rx.try_recv() {
            match event {
                FrameEvent::Frame(payload) => {
                    latest_frame = Some(payload);
                }
                FrameEvent::Ended => {
                    commands
                        .entity(entity)
                        .trigger(|e| VideoEnded { entity: e });
                }
                FrameEvent::Looped => {
                    commands
                        .entity(entity)
                        .trigger(|e| VideoLooped { entity: e });
                }
                FrameEvent::Error(reason) => {
                    commands
                        .entity(entity)
                        .trigger(|e| VideoFailed { entity: e, reason });
                }
            }
        }
        if let Some(payload) = latest_frame {
            if let Some(mut image) = images.get_mut(&output.image) {
                write_frame(&mut image, payload.size, payload.pixels);
            }
            output.size = payload.size;
            output.position_seconds = payload.pts_seconds;
        }
    }
}

pub(crate) fn on_player_removed(event: On<Remove, VideoPlayer>, mut commands: Commands) {
    let entity = event.event_target();
    commands
        .entity(entity)
        .remove::<(VideoWorker, VideoOutput, PendingVideo, SeekTo)>();
}

fn write_frame(image: &mut Image, size: UVec2, pixels: Vec<u8>) {
    let extent = Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    };
    if image.texture_descriptor.size != extent {
        image.resize(extent);
    }
    image.data = Some(pixels);
}
