use std::error::Error;

#[derive(Debug)]
pub struct Platform(crate::winit_proxy::SendEvent);

impl super::PlatformTrait for Platform {
    fn list_files(&mut self) -> Vec<String> {
        vec!["nuero.png".to_owned(), "shader.wgsl".to_owned()]
    }
    fn watch_file(&mut self, name: &str) {
        match name {
            "nuero.png" => self.0.send_event(crate::Event::FileContents(
                name.to_owned(),
                include_bytes!("../../nuero.png").to_vec(),
            )),
            "shader.wgsl" => self.0.send_event(crate::Event::FileContents(
                name.to_owned(),
                include_bytes!("../../shader.wgsl").to_vec(),
            )),
            _ => {}
        }
    }
    fn unwatch_file(&mut self, name: &str) {}
    fn new(send_event: crate::winit_proxy::SendEvent) -> Self {
        Self(send_event)
    }
    fn error_reporter(&mut self) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>) {
        |error| log::error!("{error}")
    }
}
