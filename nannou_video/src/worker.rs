use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use flume::{Receiver, Sender, TryRecvError, TrySendError};
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
    _handle: Option<thread::JoinHandle<()>>,
}

impl Drop for VideoWorker {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(PlayerCommand::Stop);
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
        _handle: Some(handle),
    }
}

struct WorkerState {
    paused: bool,
    speed: f32,
    mode: PlaybackMode,
    first_frame: bool,
    start_wall: Instant,
    start_pts: f64,
    need_keyframe: bool,
    last_pts: f64,
    consecutive_errors: u32,
}

fn worker_main(
    config: WorkerConfig,
    cmd_rx: Receiver<PlayerCommand>,
    frame_tx: Sender<FrameEvent>,
) {
    let WorkerConfig {
        source,
        mode,
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
    let mut state = WorkerState {
        paused: false,
        speed: 1.0,
        mode,
        first_frame: true,
        start_wall: Instant::now(),
        start_pts: 0.0,
        need_keyframe: false,
        last_pts: 0.0,
        consecutive_errors: 0,
    };
    let mut rgba = Vec::new();

    loop {
        if drain_commands(&cmd_rx, &mut decoder, &mut state) {
            return;
        }

        if state.paused {
            match cmd_rx.recv() {
                Ok(cmd) => {
                    if apply_command(cmd, &mut decoder, &mut state) {
                        return;
                    }
                    // After receiving a command while paused, drain any remaining
                    // commands so batched updates (Pause+SetSpeed+SetMode) settle.
                    if drain_commands(&cmd_rx, &mut decoder, &mut state) {
                        return;
                    }
                    // If still paused after draining and a seek occurred, decode
                    // one frame so the display updates to the new position.
                    if state.paused && state.need_keyframe {
                        state.need_keyframe = false;
                        if let Some(payload) =
                            decode_one_frame(&mut decoder, &mut state, &mut rgba, tb_num, tb_den)
                        {
                            let _ = frame_tx.try_send(FrameEvent::Frame(payload));
                        }
                    }
                }
                Err(_) => return,
            }
            continue;
        }

        let frame = match decoder.decode_raw() {
            Ok(f) => {
                state.consecutive_errors = 0;
                f
            }
            Err(VideoRsError::DecodeExhausted) => {
                if handle_eof(&cmd_rx, &mut decoder, &mut state, &frame_tx) {
                    return;
                }
                continue;
            }
            Err(_e) => {
                state.consecutive_errors += 1;
                if state.consecutive_errors > 3 {
                    if handle_eof(&cmd_rx, &mut decoder, &mut state, &frame_tx) {
                        return;
                    }
                }
                continue;
            }
        };

        let pts_seconds = frame
            .pts()
            .map(|p| p as f64 * tb_num / tb_den)
            .unwrap_or(0.0);
        state.last_pts = pts_seconds;

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

        if state.first_frame {
            state.start_wall = Instant::now();
            state.start_pts = pts_seconds;
            state.first_frame = false;
        } else {
            let target = (pts_seconds - state.start_pts) / state.speed.max(0.001) as f64;
            let elapsed = state.start_wall.elapsed().as_secs_f64();
            if target > elapsed {
                thread::sleep(Duration::from_secs_f64(target - elapsed));
            }
        }

        let payload = FramePayload {
            pts_seconds,
            pixels: std::mem::take(&mut rgba),
            size: UVec2::new(width, height),
        };

        // Non-blocking send: if the channel is full, drop the frame rather than
        // blocking the worker (which would prevent command processing).
        match frame_tx.try_send(FrameEvent::Frame(payload)) {
            Ok(_) => {}
            Err(TrySendError::Full(_)) => {
                state.start_wall = Instant::now();
                state.start_pts = pts_seconds;
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}

fn decode_one_frame(
    decoder: &mut video_rs::Decoder,
    state: &mut WorkerState,
    rgba: &mut Vec<u8>,
    tb_num: f64,
    tb_den: f64,
) -> Option<FramePayload> {
    let frame = decoder.decode_raw().ok()?;

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

    state.start_wall = Instant::now();
    state.start_pts = pts_seconds;
    state.first_frame = false;

    Some(FramePayload {
        pts_seconds,
        pixels: std::mem::take(rgba),
        size: UVec2::new(width, height),
    })
}

fn handle_eof(
    cmd_rx: &Receiver<PlayerCommand>,
    decoder: &mut video_rs::Decoder,
    state: &mut WorkerState,
    frame_tx: &Sender<FrameEvent>,
) -> bool {
    state.consecutive_errors = 0;
    match state.mode {
        PlaybackMode::Loop => {
            let _ = decoder.seek_to_frame(0);
            let _ = decoder.seek(0);
            state.first_frame = true;
            match frame_tx.try_send(FrameEvent::Looped) {
                Err(TrySendError::Disconnected(_)) => return true,
                _ => {}
            }
            false
        }
        PlaybackMode::Once => {
            match frame_tx.try_send(FrameEvent::Ended) {
                Err(TrySendError::Disconnected(_)) => return true,
                _ => {}
            }
            wait_for_restart(cmd_rx, decoder, state)
        }
    }
}

fn drain_commands(
    cmd_rx: &Receiver<PlayerCommand>,
    decoder: &mut video_rs::Decoder,
    state: &mut WorkerState,
) -> bool {
    loop {
        match cmd_rx.try_recv() {
            Ok(cmd) => {
                if apply_command(cmd, decoder, state) {
                    return true;
                }
            }
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => return true,
        }
    }
}

fn apply_command(
    cmd: PlayerCommand,
    decoder: &mut video_rs::Decoder,
    state: &mut WorkerState,
) -> bool {
    match cmd {
        PlayerCommand::Play => {
            state.paused = false;
            state.first_frame = true;
        }
        PlayerCommand::Pause => state.paused = true,
        PlayerCommand::Seek(s) => {
            let ms = (s * 1000.0) as i64;
            let _ = decoder.seek_to_frame(0);
            if ms > 0 {
                let _ = decoder.seek(ms);
            }
            state.first_frame = true;
            state.need_keyframe = true;
        }
        PlayerCommand::SetSpeed(s) => {
            state.speed = s;
            state.first_frame = true;
        }
        PlayerCommand::SetMode(m) => state.mode = m,
        PlayerCommand::Stop => return true,
    }
    false
}

fn wait_for_restart(
    cmd_rx: &Receiver<PlayerCommand>,
    decoder: &mut video_rs::Decoder,
    state: &mut WorkerState,
) -> bool {
    loop {
        let cmd = match cmd_rx.recv() {
            Ok(c) => c,
            Err(_) => return true,
        };
        match cmd {
            PlayerCommand::Seek(s) => {
                let ms = (s * 1000.0) as i64;
                let _ = decoder.seek_to_frame(0);
                if ms > 0 {
                    let _ = decoder.seek(ms);
                }
                state.first_frame = true;
                return false;
            }
            PlayerCommand::Play => {
                let _ = decoder.seek_to_frame(0);
                state.first_frame = true;
                state.paused = false;
                return false;
            }
            PlayerCommand::SetMode(PlaybackMode::Loop) => {
                state.mode = PlaybackMode::Loop;
                let _ = decoder.seek_to_frame(0);
                state.first_frame = true;
                return false;
            }
            PlayerCommand::Stop => return true,
            PlayerCommand::Pause => state.paused = true,
            PlayerCommand::SetSpeed(s) => state.speed = s,
            PlayerCommand::SetMode(m) => state.mode = m,
        }
    }
}
