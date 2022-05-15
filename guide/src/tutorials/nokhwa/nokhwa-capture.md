# An Intro to Nokhwa

**Tutorial Info**

- Author: [l1npengtul](https://l1n.pengtul.net)
- Required Knowledge:
    - [Anatomy of a nannou App](/tutorials/basics/anatomy-of-a-nannou-app.md)
    - [Nokhwa Introduction](/tutorials/nokhwa/nokhwa-introduction.md)
- Reading Time: 5 minutes
---

# Preface
Keep in mind that camera capture is complicated and stands on top of the steaming stack of complexity that is modern 
Operating Systems and the like. Many functions return a form of `Result<T>` containing a `NokhwaError`, you **must** deal 
with these to avoid `panic!`s.

# Initializing Nokhwa
First, you must call `nokhwa_initialize()`
```rust,no_run
fn on_initialized(x: bool) {
    println!("Initialized")
}

nokhwa_initialize(on_initialized)
```
(You can actually omit this on non-MacOS platforms)

# Querying List of Cameras
Querying the list of available cameras is easily done by calling `query_devices()`:
```rust,no_run
let devices = query_devices().unwrap();
for device in &devices {
    println!("{:?}", device)
}
```
This returns a list of `CameraInfo`s, which contain the human-readable name as well as backend-specific data and the 
Camera's Index, so you can open the device.

# Opening a device
If you want more information about a device, you need to first open the device:
```rust,no_run
let mut camera = NannouCamera::new(0, None).unwrap();
```

# Querying Available Camera Formats
### What is a camera format?
-> It is a specific Resolution + Framerate + Frame Format
- Resolutions are in format (width, height)
- Framerate is a u32 integer.
- Frame formats are either YUYV(YUY2) or MJPG.

### Querying Camera Formats
Call `NannouCamera::compatible_camera_formats`:
```rust,no_run
let frame_foramts = camera.compatible_camera_formats().unwrap();
```

# Capturing Frames from the Camera
First, you must open the camera stream:
```rust,no_run
camera.open_stream().unwrap();
```
You can check if the camera is open by using `NannouCamera::is_sream_open`
```rust,no_run
let is_open = camera.is_stream_open();
```
#### A note on `ImageTexture`:
But before we get into frame capture, the `NannouCamera` returns an `ImageTexture` every time it captures a frame. This
is a thin wrapper around `ImageBuffer<Rgb<u8>, Vec<u8>>` with utility functions that allow for easy integration with 
`nannou_wgpu`. To convert an `ImageTexture` to a  `Texture`, use either `into_texture` or `loaded_texture_with_device_and_queue`.

The `NannouCamera` implementation is analogous to `nokhwa::ThreadedCamera`, which means
you can get frames from the camera in three ways:
### Using `NannouCamera::set_callback`:
`NannouCamera::set_callback` is a method that allows you to set a callback function, this function is called every time 
a new frame is captured. The function takes a `ImageTexture`, which will be explained ahead.
Remember that this function is executed within the context of the `NannouCamera`'s thread, and that if it takes too long
you will start to drop frames. 
```rust,no_run
fn frame_callback(image: ImageTexture) {
    /*
    - snip! - 
    Put things that you would do with your image here.
    */
}

fn main() {
    /* -snip- */
    camera.set_callback(frame_callback);
    /* -snip- */
}
```
### Using `NannouCamera::last_frame`:
`NannouCamera::poll_frame` is a method that allows you to get the last captured frame. This function returns instantly 
as long as the stream is open. You can also use either `last_frame_texture` or `last_frame_texture_with_device_queue_usage` to poll
a `Texture`.
```rust,no_run
let poll_frame = camera.last_frame().unwrap();
```
### Using `NannouCamera::poll_frame`:
`NannouCamera::poll_frame` is a method that allows you to **wait** for the next frame. This function is blocking. Keep
in mind that this function competes for the frame with `set_callback` **and** `last_frame`, so it is not recommended to use.
You can also use either `poll_texture` or `poll_texture_with_device_queue_usage` to poll
a `Texture`.
```rust,no_run
let poll_frame = camera.poll_frame().unwrap();
```

# Closing the camera
You can close the camera with 
```rust,no_run
camera.stop_stream().unwrap();
```
Alternatively, it will automatically close when dropped.

