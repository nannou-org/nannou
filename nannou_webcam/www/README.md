# bevy_webcam for web

## wasm support

to build wasm run:

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ./www/out/ --target web ./target/wasm32-unknown-unknown/release/bevy_webcam.wasm
```

then serve `index.html` from a local web server (for example `cd www && python -m http.server 8080`). The page acquires a webcam stream via `MediaStreamTrackProcessor`, converts each frame to RGBA, and forwards it into the wasm module through the exported `frame_input` function.

> Note: browsers require `getUserMedia` to be served from `https://` or `http://localhost`, so opening the file directly from disk will not work.
