//! An example of reading files of the ILDA Image Data Transfer Format and playing them back with
//! nannou.
//!
//! Specify a directory containing `.ild` or `.ILD` files to play them with this example. E.g.
//!
//! ```
//! cargo run --release -p examples --example laser_ilda_idtf -- /path/to/ilda/files
//! ```

use nannou::prelude::*;
use nannou_laser as laser;
use nannou_laser::ilda_idtf::BufFileFrameReader;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() {
    nannou::app(model).run();
}

struct Model {
    _laser_api: laser::Api,
    _laser_stream: laser::FrameStream<Laser>,
}

struct Laser {
    ilda_paths: Vec<PathBuf>,
    current_path: usize,
    last_change: Instant,
    frame_reader: BufFileFrameReader,
}

fn model(app: &App) -> Model {
    // Create a window to receive keyboard events.
    app.new_window().view(view).build();

    // Test file directory from the command line.
    let test_files = match std::env::args().nth(1) {
        None => panic!("Please specify a directory with at least one '.ild' or '.ILD' file"),
        Some(s) => PathBuf::from(s),
    };

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let ilda_paths: Vec<_> = find_ilda_files(&test_files).collect();
    if ilda_paths.is_empty() {
        panic!(
            "Could not find any '.ild' or '.ILD' files in {}",
            test_files.display()
        );
    }
    let current_path = 0;
    let path = &ilda_paths[current_path];
    let last_change = Instant::now();
    let frame_reader = BufFileFrameReader::open(path).unwrap();
    let laser_model = Laser {
        ilda_paths,
        last_change,
        current_path,
        frame_reader,
    };

    // Initialise the LASER API and spawn the stream.
    let _laser_api = laser::Api::new();
    let laser_stream = _laser_api
        .new_frame_stream(laser_model, laser)
        .tcp_timeout(Some(Duration::from_secs(1)))
        // ILDA IDTF spec says files should be optimised already.
        .enable_optimisations(false)
        .build()
        .unwrap();
    laser_stream.set_point_hz(10_000).unwrap();
    laser_stream.set_frame_hz(30).unwrap();

    Model {
        _laser_api,
        _laser_stream: laser_stream,
    }
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);
}

fn laser(laser: &mut Laser, frame: &mut laser::Frame) {
    loop {
        match laser.frame_reader.next() {
            Ok(Some(points)) => {
                frame.add_lines(points);
                return;
            }
            Err(err) => {
                let path = &laser.ilda_paths[laser.current_path];
                eprintln!("error occurred while reading {}: {}", path.display(), err);
                laser.last_change = Instant::now();
                laser.current_path += 1;
            }
            Ok(None) => {
                if laser.last_change.elapsed() > Duration::from_secs(2) {
                    laser.last_change = Instant::now();
                    laser.current_path += 1;
                }
            }
        }
        laser.current_path %= laser.ilda_paths.len();
        let path = &laser.ilda_paths[laser.current_path];
        println!("{}", path.display());
        laser.frame_reader = BufFileFrameReader::open(path).unwrap();
    }
}

fn find_ilda_files(dir: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(dir).into_iter().filter_map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if ext == Some("ild") || ext == Some("ILD") {
            Some(path.to_path_buf())
        } else {
            None
        }
    })
}
