use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use flume::{Receiver, Sender, TryRecvError};
use video_rs::hwaccel::HardwareAccelerationDeviceType;
use video_rs::options::Options;
use video_rs::resize::Resize;
use video_rs::{DecoderBuilder, Error as VideoRsError};

use crate::asset::VideoSource;
use crate::components::PlaybackMode;

pub(crate) struct WorkerConfig {
    pub source: VideoSource,
    pub mode: PlaybackMode,
    pub hw_accel: Option<HardwareAccelerationDeviceType>,
    pub resize: Option<Resize>,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) enum PlayerCommand {
    Play,
    Pause,
    Seek(f64),
    SetSpeed(f32),
    SetMode(PlaybackMode),
    Stop,
}

pub(crate) struct FramePayload {
    pub pts_seconds: f64,
    pub pixels: Vec<u8>,
    pub size: UVec2,
}

pub(crate) enum FrameEvent {
    Frame(FramePayload),
    Ended,
    Looped,
    Error(String),
}

#[derive(Component)]
pub(crate) struct VideoWorker {
    pub cmd_tx: Sender<PlayerCommand>,
    pub frame_rx: Receiver<FrameEvent>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for VideoWorker {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(PlayerCommand::Stop);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

pub(crate) fn spawn_worker(config: WorkerConfig) -> VideoWorker {
    let (cmd_tx, cmd_rx) = flume::unbounded();
    let (frame_tx, frame_rx) = flume::bounded(3);
    let handle = thread::Builder::new()
        .name("nannou_video_decode".to_string())
        .spawn(move || worker_main(config, cmd_rx, frame_tx))
        .expect("failed to spawn video worker thread");
    VideoWorker {
        cmd_tx,
        frame_rx,
        handle: Some(handle),
    }
}

fn worker_main(
    config: WorkerConfig,
    cmd_rx: Receiver<PlayerCommand>,
    frame_tx: Sender<FrameEvent>,
) {
    let WorkerConfig {
        source,
        mut mode,
        hw_accel,
        resize,
        options,
    } = config;
    let options: Options = options.into();
    let mut builder = DecoderBuilder::new(source.to_location()).with_options(&options);
    if let Some(device) = hw_accel {
        builder = builder.with_hardware_acceleration(device);
    }
    if let Some(resize) = resize {
        builder = builder.with_resize(resize);
    }
    let mut decoder = match builder.build() {
        Ok(d) => d,
        Err(e) => {
            let _ = frame_tx.send(FrameEvent::Error(format!("{}", e)));
            return;
        }
    };

    let time_base = decoder.time_base();
    let tb_num = time_base.numerator() as f64;
    let tb_den = time_base.denominator() as f64;

    let mut paused = false;
    let mut speed: f32 = 1.0;
    let mut start_wall = Instant::now();
    let mut start_pts: f64 = 0.0;
    let mut first_frame = true;
    let mut rgba = Vec::new();

    loop {
        loop {
            match cmd_rx.try_recv() {
                Ok(cmd) => {
                    if apply_command(
                        cmd,
                        &mut decoder,
                        &mut paused,
                        &mut speed,
                        &mut mode,
                        &mut first_frame,
                    ) {
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        if paused {
            match cmd_rx.recv() {
                Ok(cmd) => {
                    if apply_command(
                        cmd,
                        &mut decoder,
                        &mut paused,
                        &mut speed,
                        &mut mode,
                        &mut first_frame,
                    ) {
                        return;
                    }
                }
                Err(_) => return,
            }
            continue;
        }

        let frame = match decoder.decode_raw() {
            Ok(f) => f,
            Err(VideoRsError::DecodeExhausted) => {
                match mode {
                    PlaybackMode::Loop => {
                        let _ = decoder.seek_to_start();
                        first_frame = true;
                        if frame_tx.send(FrameEvent::Looped).is_err() {
                            return;
                        }
                    }
                    PlaybackMode::Once => {
                        if frame_tx.send(FrameEvent::Ended).is_err() {
                            return;
                        }
                        if wait_for_restart(
                            &cmd_rx,
                            &mut decoder,
                            &mut paused,
                            &mut speed,
                            &mut mode,
                            &mut first_frame,
                        ) {
                            return;
                        }
                    }
                }
                continue;
            }
            Err(e) => {
                let _ = frame_tx.send(FrameEvent::Error(format!("{}", e)));
                continue;
            }
        };

        let pts_seconds = frame
            .pts()
            .map(|p| p as f64 * tb_num / tb_den)
            .unwrap_or(0.0);

        let width = frame.width();
        let height = frame.height();
        let stride = frame.stride(0);
        let src = frame.data(0);
        let row_rgb = (width as usize) * 3;
        let required = (width as usize) * (height as usize) * 4;
        rgba.clear();
        rgba.reserve(required);
        for y in 0..height as usize {
            let row_start = y * stride;
            let row = &src[row_start..row_start + row_rgb];
            for rgb in row.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }

        if first_frame {
            start_wall = Instant::now();
            start_pts = pts_seconds;
            first_frame = false;
        } else {
            let target = (pts_seconds - start_pts) / speed.max(0.001) as f64;
            let elapsed = start_wall.elapsed().as_secs_f64();
            if target > elapsed {
                thread::sleep(Duration::from_secs_f64(target - elapsed));
            }
        }

        let payload = FramePayload {
            pts_seconds,
            pixels: std::mem::take(&mut rgba),
            size: UVec2::new(width, height),
        };
        let before_send = Instant::now();
        if frame_tx.send(FrameEvent::Frame(payload)).is_err() {
            return;
        }
        if before_send.elapsed() > Duration::from_millis(5) {
            start_wall = Instant::now();
            start_pts = pts_seconds;
        }
    }
}

fn apply_command(
    cmd: PlayerCommand,
    decoder: &mut video_rs::Decoder,
    paused: &mut bool,
    speed: &mut f32,
    mode: &mut PlaybackMode,
    first_frame: &mut bool,
) -> bool {
    match cmd {
        PlayerCommand::Play => {
            *paused = false;
            *first_frame = true;
        }
        PlayerCommand::Pause => *paused = true,
        PlayerCommand::Seek(s) => {
            let _ = decoder.seek((s * 1000.0) as i64);
            *first_frame = true;
        }
        PlayerCommand::SetSpeed(s) => {
            *speed = s;
            *first_frame = true;
        }
        PlayerCommand::SetMode(m) => *mode = m,
        PlayerCommand::Stop => return true,
    }
    false
}

fn wait_for_restart(
    cmd_rx: &Receiver<PlayerCommand>,
    decoder: &mut video_rs::Decoder,
    paused: &mut bool,
    speed: &mut f32,
    mode: &mut PlaybackMode,
    first_frame: &mut bool,
) -> bool {
    loop {
        let cmd = match cmd_rx.recv() {
            Ok(c) => c,
            Err(_) => return true,
        };
        match cmd {
            PlayerCommand::Seek(s) => {
                let _ = decoder.seek((s * 1000.0) as i64);
                *first_frame = true;
                return false;
            }
            PlayerCommand::Play => {
                let _ = decoder.seek_to_start();
                *first_frame = true;
                *paused = false;
                return false;
            }
            PlayerCommand::SetMode(PlaybackMode::Loop) => {
                *mode = PlaybackMode::Loop;
                let _ = decoder.seek_to_start();
                *first_frame = true;
                return false;
            }
            PlayerCommand::Stop => return true,
            PlayerCommand::Pause => *paused = true,
            PlayerCommand::SetSpeed(s) => *speed = s,
            PlayerCommand::SetMode(m) => *mode = m,
        }
    }
}
