//! An example of using `nannou_laser`'s `ffi`.
//!
//! While this example is in rust and not C, it should give an idea how to use the API via C.

#[repr(C)]
struct CallbackData {
    pattern: u32,
}

type Pattern = u32;
const RECTANGLE: Pattern = 0;
const TRIANGLE: Pattern = 1;
const CROSSHAIR: Pattern = 2;
const THREE_VERTICAL_LINES: Pattern = 3;
const SPIRAL: Pattern = 4;
const TOTAL_PATTERNS: u32 = SPIRAL + 1;

fn main() {
    unsafe {
        // Create the API.
        println!("Initialising API...");
        let mut api = std::ptr::null_mut();
        nannou_laser::ffi::api_new(&mut api);

        // Detect DAC.
        println!("Detecting DAC...");
        let mut dac = std::mem::MaybeUninit::<nannou_laser::ffi::DetectedDac>::uninit();
        let res = nannou_laser::ffi::detect_dac(api, dac.as_mut_ptr());
        if res as u32 != 0 {
            eprintln!("failed to detect DAC");
            nannou_laser::ffi::api_drop(api);
            return;
        }
        let dac = dac.assume_init();
        println!("Found DAC!");

        // Only ether dream supported presently.
        let ether_dream = dac.kind.ether_dream;
        println!("{:#?}", ether_dream);

        // Create a frame stream.
        let mut frame_stream_conf =
            std::mem::MaybeUninit::<nannou_laser::ffi::FrameStreamConfig>::uninit();
        nannou_laser::ffi::frame_stream_config_default(frame_stream_conf.as_mut_ptr());
        let frame_stream_conf = frame_stream_conf.assume_init();
        let callback_data: *mut CallbackData =
            Box::into_raw(Box::new(CallbackData { pattern: RECTANGLE }));
        let mut frame_stream = std::ptr::null_mut();
        println!("Spawning new frame stream...");
        nannou_laser::ffi::new_frame_stream(
            api,
            &mut frame_stream,
            &frame_stream_conf,
            callback_data as *mut std::os::raw::c_void,
            frame_render_callback,
            process_raw_callback,
        );

        // Run through each pattern for 1 second.
        for pattern in 0..TOTAL_PATTERNS {
            println!("drawing pattern {}", pattern);
            (*callback_data).pattern = pattern;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // Drop frame stream to close it.
        println!("Dropping the frame stream");
        nannou_laser::ffi::frame_stream_drop(frame_stream);
        std::mem::drop(Box::from_raw(callback_data));

        // Create a raw stream.
        let mut stream_conf = std::mem::MaybeUninit::<nannou_laser::ffi::StreamConfig>::uninit();
        nannou_laser::ffi::stream_config_default(stream_conf.as_mut_ptr());
        let stream_conf = stream_conf.assume_init();
        let callback_data: *mut CallbackData =
            Box::into_raw(Box::new(CallbackData { pattern: RECTANGLE }));
        let mut raw_stream = std::ptr::null_mut();
        println!("Spawning a raw stream...");
        nannou_laser::ffi::new_raw_stream(
            api,
            &mut raw_stream,
            &stream_conf,
            callback_data as *mut std::os::raw::c_void,
            raw_render_callback,
        );

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Drop the raw stream to close it.
        println!("Dropping the raw stream");
        nannou_laser::ffi::raw_stream_drop(raw_stream);
        std::mem::drop(Box::from_raw(callback_data));

        // Release the handle to the API when we're done.
        println!("Cleaning up...");
        nannou_laser::ffi::api_drop(api);

        println!("Done!");
    }
}

// Called when the stream is ready for a new frame of data.
extern "C" fn frame_render_callback(
    data: *mut std::os::raw::c_void,
    frame: *mut nannou_laser::ffi::Frame,
) {
    unsafe {
        let data_ptr = data as *mut CallbackData;
        let data = &mut *data_ptr;
        let frame = &mut *frame;
        write_laser_frame_points(data, frame);
    }
}

fn write_laser_frame_points(data: &mut CallbackData, frame: &mut nannou_laser::ffi::Frame) {
    // Simple constructors for white or blank points.
    let lit_p = |position| nannou_laser::Point::new(position, [1.0; 3]);

    // Draw the frame with the selected pattern.
    match data.pattern {
        RECTANGLE => {
            let tl = [-1.0, 1.0];
            let tr = [1.0, 1.0];
            let br = [1.0, -1.0];
            let bl = [-1.0, -1.0];
            let positions = [tl, tr, br, bl, tl];
            let points: Vec<_> = positions.iter().cloned().map(lit_p).collect();
            unsafe {
                nannou_laser::ffi::frame_add_lines(frame, points.as_ptr(), points.len());
            }
        }

        TRIANGLE => {
            let a = [-0.75, -0.75];
            let b = [0.0, 0.75];
            let c = [0.75, -0.75];
            let positions = [a, b, c, a];
            let points: Vec<_> = positions.iter().cloned().map(lit_p).collect();
            unsafe {
                nannou_laser::ffi::frame_add_lines(frame, points.as_ptr(), points.len());
            }
        }

        CROSSHAIR => {
            let xa = [-1.0, 0.0];
            let xb = [1.0, 0.0];
            let ya = [0.0, -1.0];
            let yb = [0.0, 1.0];
            let x = [lit_p(xa), lit_p(xb)];
            let y = [lit_p(ya), lit_p(yb)];
            unsafe {
                nannou_laser::ffi::frame_add_lines(frame, x.as_ptr(), x.len());
                nannou_laser::ffi::frame_add_lines(frame, y.as_ptr(), y.len());
            }
        }

        THREE_VERTICAL_LINES => {
            let la = [-1.0, -0.5];
            let lb = [-1.0, 0.5];
            let ma = [0.0, 0.5];
            let mb = [0.0, -0.5];
            let ra = [1.0, -0.5];
            let rb = [1.0, 0.5];
            let l = [lit_p(la), lit_p(lb)];
            let m = [lit_p(ma), lit_p(mb)];
            let r = [lit_p(ra), lit_p(rb)];
            unsafe {
                nannou_laser::ffi::frame_add_lines(frame, l.as_ptr(), l.len());
                nannou_laser::ffi::frame_add_lines(frame, m.as_ptr(), m.len());
                nannou_laser::ffi::frame_add_lines(frame, r.as_ptr(), r.len());
            }
        }

        SPIRAL => {
            let n_points = unsafe { nannou_laser::ffi::points_per_frame(frame) as usize / 2 };
            let radius = 1.0;
            let rings = 5.0;
            let points = (0..n_points)
                .map(|i| {
                    let fract = i as f32 / n_points as f32;
                    let mag = fract * radius;
                    let phase = rings * fract * 2.0 * std::f32::consts::PI;
                    let y = mag * -phase.sin();
                    let x = mag * phase.cos();
                    [x, y]
                })
                .map(lit_p)
                .collect::<Vec<_>>();
            unsafe {
                nannou_laser::ffi::frame_add_lines(frame, points.as_ptr(), points.len());
            }
        }

        _ => unreachable!(),
    }
}

// Called when the stream is ready for data. This is called after the `frame_render_callback` and
// after all path optimisations have been applied. This is useful as a kind of post-processing
// function, for applying safety zones, etc.
extern "C" fn process_raw_callback(
    _data: *mut std::os::raw::c_void,
    _buffer: *mut nannou_laser::ffi::Buffer,
) {
}

// Called when then
extern "C" fn raw_render_callback(
    _data: *mut std::os::raw::c_void,
    _buffer: *mut nannou_laser::ffi::Buffer,
) {
}
