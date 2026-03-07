# bevy_webcam 📷

[![GitHub License](https://img.shields.io/github/license/mosure/bevy_webcam)](https://raw.githubusercontent.com/mosure/bevy_webcam/main/LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/bevy_webcam.svg)](https://crates.io/crates/bevy_webcam)

bevy camera input, using the nokhwa crate


## usage

```rust
app.add_plugins((
    DefaultPlugins,
    BevyWebcamPlugin::default(),
));
app.add_systems(
    Update,
    setup_ui,
);

// ...

fn setup_ui(
    mut commands: Commands,
    stream: Res<WebcamStream>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        ImageNode {
            image: stream.frame.clone(),
            ..default()
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}
```


## features

- [x] native camera capture via `nokhwa`'s native backends
- [x] threaded frame decoding on native targets, so the Bevy `Update` stage stays responsive
- [x] wasm32 (browser) capture via the DOM `MediaStreamTrackProcessor` feeding pixels into the exported `frame_input` binding

## platform notes

- **Native:** frames are decoded on a dedicated worker thread and sent to the main Bevy world through a channel before being uploaded to the GPU.
- **Wasm:** `www/index.html` acquires the webcam stream with `getUserMedia`, processes frames with `MediaStreamTrackProcessor`, and forwards RGBA pixels into the wasm module via `frame_input`. The Bevy plugin simply consumes those frames each `Update`, so there is no blocking `nokhwa` path on the browser.
- **Camera selection on web:** the browser decides which device backs the stream the user grants; the `CameraIndex` setting currently applies to native builds only.
