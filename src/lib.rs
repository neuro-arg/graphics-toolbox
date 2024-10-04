use std::{borrow::Cow, future::Future};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Debug)]
struct InitApp {
    config: SurfaceConfiguration,
    surface: Surface<'static>,
    device: Device,
    window: &'static Window,
    render_pipeline: RenderPipeline,
    queue: Queue,
}

#[derive(Debug)]
enum App {
    Uninit(EventLoopProxy<InitApp>),
    Waiting {
        events: Vec<(WindowId, WindowEvent)>,
    },
    Init(InitApp),
}

impl InitApp {
    #[cfg(target_arch = "wasm32")]
    fn window_attrs() -> WindowAttributes {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        Self::base_window_attrs()
            .with_canvas(Some(canvas))
            .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn window_attrs() -> WindowAttributes {
        Self::base_window_attrs()
    }
    fn base_window_attrs() -> WindowAttributes {
        winit::window::WindowAttributes::default()
    }
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> impl 'static + Future<Output = Self> {
        let window = event_loop.create_window(Self::window_attrs()).unwrap();
        let mut size = window.inner_size();
        size.width = size.width.max(640);
        size.height = size.height.max(480);
        let instance = wgpu::Instance::default();

        async move {
            // XXX: I hate this
            let window = Box::leak(Box::new(window));
            let surface = instance.create_surface(&*window).unwrap();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    // Request an adapter which can render to our surface
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Failed to find an appropriate adapter");

            // Create the logical device and command queue
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                        required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                            .using_resolution(adapter.limits()),
                        memory_hints: wgpu::MemoryHints::MemoryUsage,
                    },
                    None,
                )
                .await
                .expect("Failed to create device");

            // Load the shaders from disk
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shader.wgsl"))),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let swapchain_capabilities = surface.get_capabilities(&adapter);
            let swapchain_format = swapchain_capabilities.formats[0];

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

            let config = surface
                .get_default_config(&adapter, size.width, size.height)
                .unwrap();
            surface.configure(&device, &config);
            Self {
                config,
                device,
                queue,
                render_pipeline,
                surface,
                window,
            }
        }
    }
}

impl ApplicationHandler for InitApp {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                // Reconfigure the surface with the new size
                self.config.width = new_size.width.max(1);
                self.config.height = new_size.height.max(1);
                self.surface.configure(&self.device, &self.config);
                // On macos the window needs to be redrawn manually after resizing
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = self
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(&self.render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                self.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        };
    }
}
impl ApplicationHandler<InitApp> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            Self::Uninit(_) => {
                let mut tmp = Self::Waiting { events: Vec::new() };
                std::mem::swap(&mut tmp, self);
                let Self::Uninit(proxy) = tmp else {
                    unreachable!()
                };
                let app = InitApp::new(event_loop);
                spawn_future(async move {
                    let app = app.await;
                    proxy
                        .send_event(app)
                        .map_err(|_| "event loop closed")
                        .unwrap();
                });
            }
            Self::Waiting { .. } => (),
            Self::Init(x) => x.resumed(event_loop),
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match self {
            Self::Waiting { events } => events.push((window_id, event)),
            Self::Init(this) => {
                this.window_event(event_loop, window_id, event);
            }
            Self::Uninit(..) => {}
        }
    }
    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Self::Init(this) = self else {
            return;
        };
        this.exiting(event_loop)
    }
    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Self::Init(this) = self else {
            return;
        };
        this.suspended(event_loop)
    }
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let Self::Init(this) = self else {
            return;
        };
        this.new_events(event_loop, cause)
    }
    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Self::Init(this) = self else {
            return;
        };
        this.device_event(event_loop, device_id, event)
    }
    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Self::Init(this) = self else {
            return;
        };
        this.about_to_wait(event_loop)
    }
    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Self::Init(this) = self else {
            return;
        };
        this.memory_warning(event_loop)
    }
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, app: InitApp) {
        let mut this = Self::Init(app);
        std::mem::swap(&mut this, self);
        let Self::Waiting { events } = this else {
            return;
        };
        for (window_id, event) in events {
            self.window_event(event_loop, window_id, event)
        }
    }
}

async fn run(event_loop: EventLoop<InitApp>) {
    let proxy = event_loop.create_proxy();
    event_loop.run_app(&mut App::Uninit(proxy)).unwrap();
}

fn spawn_future<F: 'static + Future<Output = ()>>(f: F) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(f);
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(f);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }

    let event_loop = EventLoop::with_user_event().build().unwrap();
    spawn_future(run(event_loop));
}
