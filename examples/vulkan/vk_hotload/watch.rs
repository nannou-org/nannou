use std::sync::mpsc;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use crate::shader::compile_shader;

pub struct Handler {
    thread_tx: mpsc::Sender<()>,
    handle: Option<thread::JoinHandle<()>>,
    _watcher: RecommendedWatcher,
}

pub enum ShaderMsg {
    Vert(Vec<u32>),
    Frag(Vec<u32>),
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.thread_tx.send(()).ok();
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
    }
}


pub fn new(vert_path: &PathBuf, frag_path: &PathBuf) -> (Handler, mpsc::Receiver<ShaderMsg>) {
    let (notify_tx, notify_rx) = mpsc::channel();
    let (thread_tx, thread_rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher =
        Watcher::new(notify_tx, Duration::from_millis(50)).expect("failed to create watcher");

    watcher
        .watch(vert_path, RecursiveMode::NonRecursive)
        .expect("failed to add vertex shader to notify");
    watcher
        .watch(frag_path, RecursiveMode::NonRecursive)
        .expect("failed to add fragment shader to notify");

    let (shader_tx, shader_rx) = mpsc::channel();

    let handle = thread::spawn(move || 'watch_loop: loop {
        if let Ok(_) = thread_rx.try_recv() {
            break 'watch_loop;
        }
        if let Ok(notify::DebouncedEvent::Create(p)) = notify_rx.recv_timeout(Duration::from_secs(1)) {
            if p.ends_with("hotload_vert.glsl") {
                match compile_shader(p.clone(), shaderc::ShaderKind::Vertex) {
                    Ok(v) => { shader_tx.send(ShaderMsg::Vert(v)).ok(); },
                    Err(e) => eprintln!("Compile Error: {:?}", e),
                }
            }
            if p.ends_with("hotload_frag.glsl") {
                match compile_shader(p.clone(), shaderc::ShaderKind::Fragment) {
                    Ok(v) => { shader_tx.send(ShaderMsg::Frag(v)).ok(); },
                    Err(e) => eprintln!("Compile Error: {:?}", e),
                }
            }
        }
    });
    let handle = Some(handle);
    let shader_watch = Handler {
        thread_tx,
        handle,
        _watcher: watcher,
    };
    (shader_watch, shader_rx)
}
