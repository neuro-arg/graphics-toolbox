use std::{collections::BTreeSet, sync::Mutex};

static FILE_LIST: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
static FILE_LIST: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

#[derive(Debug)]
pub struct Platform;

impl super::PlatformTrait for Platform {
    type FileData = Vec<u8>;
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
    fn load_file_if_changed(&mut self, name: &str) -> Option<Self::FileData> {
        self.0
            .lock()
            .unwrap()
            .remove(name)
            .then(|| fs::read(name).ok())
            .flatten()
    }
    fn new(mut send_event: impl 'static + Send + Sync + FnMut(crate::Event)) -> Self {
        let mut ret = Self(Arc::default());
        let files = ret.list_files();
        ret.0.lock().unwrap().extend(files);
        let arc = Arc::downgrade(&ret.0);
        std::thread::spawn(move || {
            let (done_tx, done_rx) = std::sync::mpsc::sync_channel(1);
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
                for path in event.paths {
                    if let Some(path) = path.file_name().and_then(|name| name.to_str()) {
                        arc.insert(path.to_owned());
                    }
                }
                send_event(crate::Event::Redraw);
            })
            .unwrap();

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher
                .watch(Path::new("."), RecursiveMode::NonRecursive)
                .unwrap();
            let _ = done_rx.recv();
        });
        ret
    }
    fn error_reporter(
        &mut self,
    ) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Send + Sync + Error>) {
        |error| log::error!("{error}")
    }
}
