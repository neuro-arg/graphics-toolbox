use std::{
    collections::HashSet,
    error::Error,
    future::Future,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

use notify::{RecursiveMode, Watcher};
use winit::window::WindowAttributes;

// very bad impl for testing and stuff
#[derive(Debug)]
pub struct Platform(
    Arc<Mutex<HashSet<String>>>,
    mpsc::SyncSender<(String, bool)>,
);

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
        self.1.send((name.to_owned(), true)).unwrap()
    }
    fn unwatch_file(&mut self, name: &str) {
        self.1.send((name.to_owned(), false)).unwrap()
    }
    fn new(send_event: crate::winit_proxy::SendEvent) -> Self {
        let (watch_tx, watch_rx) = mpsc::sync_channel(16);
        let mut ret = Self(Arc::default(), watch_tx);
        let files = ret.list_files();
        ret.0.lock().unwrap().extend(files);
        let arc = Arc::downgrade(&ret.0);
        std::thread::spawn(move || {
            let (done_tx, done_rx) = std::sync::mpsc::sync_channel(1);
            let send_event1 = send_event.clone();
            let mut watcher = notify::recommended_watcher(move |res: Result<_, _>| {
                let event: notify::Event = res.unwrap();
                match event.kind {
                    notify::EventKind::Access(notify::event::AccessKind::Close(
                        notify::event::AccessMode::Write,
                    )) => {}
                    _ => return,
                }
                let Some(arc) = arc.upgrade() else {
                    let _ = done_tx.send(());
                    return;
                };
                let mut arc = arc.lock().unwrap();
                for path in &event.paths {
                    if let Some(path) = path.file_name().and_then(|name| name.to_str()) {
                        arc.insert(path.to_owned());
                    }
                }
                drop(arc);
                for path in &event.paths {
                    if let Some(path) = path.file_name().and_then(|name| name.to_str()) {
                        if let Ok(contents) = std::fs::read(path) {
                            send_event1
                                .send_event(crate::Event::FileContents(path.to_owned(), contents));
                        }
                    }
                }
            })
            .unwrap();

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            while let Ok(x) = watch_rx.recv() {
                match x {
                    (x, true) => {
                        if let Ok(contents) = std::fs::read(&x) {
                            send_event.send_event(crate::Event::FileContents(x.clone(), contents));
                        }
                        watcher.watch(Path::new(&x), RecursiveMode::NonRecursive)
                    }
                    (x, false) => watcher.unwatch(Path::new(&x)),
                }
                .unwrap()
            }
            let _ = done_rx.recv();
        });
        ret
    }
    fn error_reporter(&mut self) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>) {
        |error| log::error!("{error}")
    }
}
