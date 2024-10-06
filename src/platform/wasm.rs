use std::error::Error;
use super::SendEvent;

#[derive(Debug)]
pub struct Platform;

impl super::PlatformTrait for Platform {
    fn list_files(&mut self) -> Vec<String> {
        vec![]
    }
    fn watch_file(&mut self, _name: &str) {
    }
    fn unwatch_file(&mut self, _name: &str) {
    }
    fn new(_send_event: impl SendEvent) -> Self {
        Self
    }
    fn error_reporter(
        &mut self,
    ) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>) {
        |error| log::error!("{error}")
    }
}
