use std::{
    collections::{HashMap, HashSet},
    error::Error,
    future::Future,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

use notify::{RecursiveMode, Watcher};
use winit::window::WindowAttributes;

// very bad impl for testing and stuff
#[derive(Debug)]
pub struct Platform(mpsc::SyncSender<(String, bool)>);

impl super::PlatformTrait for Platform {
    fn init() {
        env_logger::init();
    }
    fn run_future<F: 'static + Future<Output = ()>>(f: F) {
        pollster::block_on(f);
    }
    fn set_window_attrs(attrs: WindowAttributes) -> WindowAttributes {
        attrs
    }
    fn list_files(&mut self) -> Vec<String> {
        std::fs::read_dir(".")
            .into_iter()
            .flat_map(|entry| entry.into_iter())
            .flatten()
            .flat_map(|entry| entry.metadata().map(|meta| (meta, entry.file_name())))
            .filter_map(|(meta, name)| meta.is_file().then_some(name))
            .flat_map(|name| name.to_str().map(|s| s.to_owned()))
            .collect()
    }
    fn watch_file(&mut self, name: &str) {
        self.0.send((name.to_owned(), true)).unwrap()
    }
    fn unwatch_file(&mut self, name: &str) {
        self.0.send((name.to_owned(), false)).unwrap()
    }
    fn new(send_event: crate::winit_proxy::SendEvent) -> Self {
        let (watch_tx, watch_rx) = mpsc::sync_channel(16);
        let (watch_tx2, watch_rx2) = mpsc::sync_channel(16);
        let (done_tx, done_rx) = std::sync::mpsc::sync_channel::<()>(1);
        let ret = Self(watch_tx2);
        let send_event1 = send_event.clone();
        std::thread::spawn(move || {
            while let Ok((a, b)) = watch_rx2.recv() {
                if b {
                    if let Ok(contents) = std::fs::read(&a) {
                        send_event1.send_event(crate::Event::FileContents(a.clone(), contents));
                    }
                }
                if watch_tx.send((a, b)).is_err() {
                    break;
                }
            }
            drop(done_tx);
        });
        std::thread::spawn(move || {
            let send_event1 = send_event.clone();
            let mut watchers = HashSet::new();
            let mut watcher = notify::recommended_watcher(move |res: Result<_, _>| {
                let event: notify::Event = res.unwrap();
                while let Ok(x) = watch_rx.try_recv() {
                    match x {
                        (x, true) => {
                            watchers.insert(x);
                        }
                        (x, false) => {
                            watchers.remove(&x);
                        }
                    }
                }
                #[allow(clippy::single_match)]
                match event.kind {
                    notify::EventKind::Access(notify::event::AccessKind::Close(
                        notify::event::AccessMode::Write,
                    )) => {
                        for path in &event.paths {
                            if let Some(path) = path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .filter(|s| watchers.contains(*s))
                            {
                                if let Ok(contents) = std::fs::read(path) {
                                    println!("sending {path:?}");
                                    send_event1.send_event(crate::Event::FileContents(
                                        path.to_owned(),
                                        contents,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            })
            .unwrap();

            watcher
                .watch(Path::new("."), RecursiveMode::NonRecursive)
                .unwrap();
            let _ = done_rx.recv();
        });
        ret
    }
    fn error_reporter(&mut self) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>) {
        |error| log::error!("{error}")
    }
}
