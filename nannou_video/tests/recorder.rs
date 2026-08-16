#![cfg(not(target_arch = "wasm32"))]

use nannou_video::{RecorderPixelFormat, VideoRecorder, VideoRecorderConfig};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const FPS: f64 = 30.0;
const FRAMES: u64 = 30;

fn gradient_frame(index: u64, bpp: usize, bgra: bool) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(WIDTH as usize * HEIGHT as usize * bpp);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let r = (x * 255 / WIDTH) as u8;
            let g = (y * 255 / HEIGHT) as u8;
            let b = ((index * 8) % 256) as u8;
            match (bpp, bgra) {
                (3, _) => pixels.extend_from_slice(&[r, g, b]),
                (4, false) => pixels.extend_from_slice(&[r, g, b, 255]),
                (4, true) => pixels.extend_from_slice(&[b, g, r, 255]),
                _ => unreachable!(),
            }
        }
    }
    pixels
}

#[test]
fn encode_decode_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.mp4");

    let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS);
    let mut recorder = VideoRecorder::new(&path, config).unwrap();
    for i in 0..FRAMES {
        recorder.record_frame(gradient_frame(i, 4, false)).unwrap();
    }
    let encoded = recorder.finish().unwrap();
    assert_eq!(encoded, FRAMES);

    let mut decoder = video_rs::Decoder::new(path.as_path()).unwrap();
    assert_eq!(decoder.size(), (WIDTH, HEIGHT));
    let time_base = decoder.time_base();
    let (tb_num, tb_den) = (time_base.numerator() as f64, time_base.denominator() as f64);

    let mut decoded = 0u64;
    let mut last_pts = 0.0f64;
    loop {
        match decoder.decode_raw() {
            Ok(frame) => {
                if let Some(pts) = frame.pts() {
                    last_pts = pts as f64 * tb_num / tb_den;
                }
                decoded += 1;
            }
            Err(video_rs::Error::DecodeExhausted | video_rs::Error::ReadExhausted) => break,
            Err(e) => panic!("decode error after {decoded} frames: {e}"),
        }
    }
    assert_eq!(decoded, FRAMES);
    let expected_last = (FRAMES - 1) as f64 / FPS;
    assert!(
        (last_pts - expected_last).abs() < 1.5 / FPS,
        "last pts {last_pts} should be near {expected_last}"
    );
}

#[test]
fn bgra_and_rgb_inputs() {
    let dir = tempfile::tempdir().unwrap();
    for (name, format, bpp, bgra) in [
        ("bgra.mp4", RecorderPixelFormat::Bgra8, 4, true),
        ("rgb.mp4", RecorderPixelFormat::Rgb8, 3, false),
    ] {
        let path = dir.path().join(name);
        let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS).with_pixel_format(format);
        let mut recorder = VideoRecorder::new(&path, config).unwrap();
        for i in 0..5 {
            recorder.record_frame(gradient_frame(i, bpp, bgra)).unwrap();
        }
        assert_eq!(recorder.finish().unwrap(), 5);
        assert!(path.exists());
    }
}

#[test]
fn rejects_wrong_frame_size() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.mp4");
    let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS);
    let mut recorder = VideoRecorder::new(&path, config).unwrap();
    let err = recorder.record_frame(vec![0u8; 16]).unwrap_err();
    assert!(matches!(
        err,
        nannou_video::VideoRecordError::FrameSize { .. }
    ));
}

#[test]
fn rejects_invalid_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.mp4");
    assert!(VideoRecorder::new(&path, VideoRecorderConfig::new(0, HEIGHT, FPS)).is_err());
    assert!(VideoRecorder::new(&path, VideoRecorderConfig::new(WIDTH, HEIGHT, 0.0)).is_err());
    assert!(
        VideoRecorder::new(&path, VideoRecorderConfig::new(WIDTH, HEIGHT, 2_000_000.0)).is_err()
    );
}

#[test]
fn zero_frames_is_an_error_and_removes_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.mp4");
    let recorder = VideoRecorder::new(&path, VideoRecorderConfig::new(WIDTH, HEIGHT, FPS)).unwrap();
    let err = recorder.finish().unwrap_err();
    assert!(matches!(err, nannou_video::VideoRecordError::NoFrames));
    assert!(!path.exists());
}

#[test]
fn crf_controls_output_size() {
    let dir = tempfile::tempdir().unwrap();
    let mut sizes = Vec::new();
    for (name, crf) in [("hi.mp4", 18u8), ("lo.mp4", 40u8)] {
        let path = dir.path().join(name);
        let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS).with_crf(crf);
        let mut recorder = VideoRecorder::new(&path, config).unwrap();
        // Incompressible noise, so quantization dominates the file size.
        let mut rng: u32 = 1;
        for _ in 0..FRAMES {
            let mut pixels = Vec::with_capacity(WIDTH as usize * HEIGHT as usize * 4);
            for _ in 0..(WIDTH * HEIGHT) {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let [a, b, c, _] = rng.to_le_bytes();
                pixels.extend_from_slice(&[a, b, c, 255]);
            }
            recorder.record_frame(pixels).unwrap();
        }
        recorder.finish().unwrap();
        sizes.push(std::fs::metadata(&path).unwrap().len());
    }
    assert!(
        sizes[0] > sizes[1] * 2,
        "crf 18 ({}) should be much larger than crf 40 ({})",
        sizes[0],
        sizes[1]
    );
}
