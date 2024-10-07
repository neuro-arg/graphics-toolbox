use std::{error::Error, future::Future, str::FromStr};

use js_sys::{wasm_bindgen::JsCast, JsString};
use wasm_bindgen::prelude::*;
use winit::{platform::web::WindowAttributesExtWebSys, window::WindowAttributes};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object, js_name = Platform)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type JsPlatform;
    #[wasm_bindgen(method, structural, js_class = "Platform", js_name = listFiles)]
    pub fn list_files(this: &JsPlatform) -> Vec<String>;
    #[wasm_bindgen(method, structural, js_class = "Platform", js_name = watchFile)]
    pub fn watch_file(
        this: &JsPlatform,
        file: &str,
        cb: &wasm_bindgen::closure::Closure<dyn FnMut(String, js_sys::Uint8Array)>,
    );
    #[wasm_bindgen(method, structural, js_class = "Platform", js_name = unwatchFile)]
    pub fn unwatch_file(this: &JsPlatform, file: &str);
    #[wasm_bindgen(method, structural, js_class = "Platform", js_name = reportError)]
    pub fn report_error(this: &JsPlatform, error: &str);
}

#[wasm_bindgen(start)]
fn start() {
    crate::start();
}

#[derive(Debug)]
pub struct Platform(crate::winit_proxy::SendEvent, JsPlatform);

impl super::PlatformTrait for Platform {
    fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }
    fn run_future<F: 'static + Future<Output = ()>>(f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
    fn set_window_attrs(attrs: WindowAttributes) -> WindowAttributes {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        attrs
            .with_canvas(Some(canvas))
            .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
    }
    fn list_files(&mut self) -> Vec<String> {
        self.1.list_files()
    }
    fn watch_file(&mut self, name: &str) {
        log::info!("watch {name}");
        match name {
            "nuero.png" => self.0.send_event(crate::Event::FileContents(
                name.to_owned(),
                include_bytes!("../../nuero.png").to_vec(),
            )),
            _ => {
                let x = self.0.clone();
                self.1.watch_file(
                    name,
                    // FIXME
                    Box::leak(Box::new(wasm_bindgen::closure::Closure::new(
                        move |a: String, b: js_sys::Uint8Array| {
                            x.send_event(crate::Event::FileContents(a, b.to_vec()));
                        },
                    ))),
                );
            }
        }
    }
    fn unwatch_file(&mut self, name: &str) {
        log::info!("unwatch {name}");
        self.1.unwatch_file(name);
    }
    fn new(send_event: crate::winit_proxy::SendEvent) -> Self {
        log::info!("new");
        Self(
            send_event,
            JsPlatform::unchecked_from_js(
                web_sys::window().unwrap().get("platform").unwrap().into(),
            ),
        )
    }
    fn error_reporter(&mut self) -> impl 'static + Send + Sync + Fn(Box<dyn 'static + Error>) {
        |error| log::error!("{error}")
    }
}
