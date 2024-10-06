use std::{error::Error, fmt::Debug};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub trait PlatformTrait: Debug {
    fn new(send_event: crate::winit_proxy::SendEvent) -> Self;
    fn watch_file(&mut self, name: &str);
    #[allow(dead_code)]
    fn unwatch_file(&mut self, name: &str);
    fn list_files(&mut self) -> Vec<String>;
    fn error_reporter(&mut self) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>);
}

#[cfg(target_arch = "wasm32")]
pub type Platform = wasm::Platform;
#[cfg(not(target_arch = "wasm32"))]
pub type Platform = native::Platform;
