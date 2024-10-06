use std::{error::Error, fmt::Debug};

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;

pub trait SendEvent: 'static + Send + Sync + Clone {
    fn send_event(&self, event: crate::Event);
}

pub trait PlatformTrait: Debug {
    type FileData: AsRef<[u8]>;
    fn new(send_event: impl SendEvent) -> Self;
    fn watch_file(&mut self, name: &str);
    #[allow(dead_code)]
    fn unwatch_file(&mut self, name: &str);
    fn list_files(&mut self) -> Vec<String>;
    fn error_reporter(
        &mut self,
    ) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Send + Sync + Error>);
}

#[cfg(target_arch = "wasm32")]
pub type Platform = wasm::Platform;
#[cfg(not(target_arch = "wasm32"))]
pub type Platform = native::Platform;
