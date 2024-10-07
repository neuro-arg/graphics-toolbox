#![allow(clippy::single_match)]
use core::str;
use image::GenericImageView;
use std::{borrow::Cow, future::Future};
use wgpu::{
    Adapter, BindGroup, BindGroupLayout, BufferUsages, Device, Queue, RenderPipeline, Surface,
    SurfaceConfiguration,
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};
use winit_proxy::WinitProxy;

use platform::{Platform, PlatformTrait};

mod platform;
mod winit_proxy;

#[derive(Debug)]
struct App {
    config: SurfaceConfiguration,
    surface: Surface<'static>,
    device: Device,
    adapter: Adapter,
    window: &'static Window,
    render_pipeline: Option<RenderPipeline>,
    queue: Queue,
    layout: BindGroupLayout,
    // platform-specific code
    _platform: Platform,
    // stuff to load/reload later
    bind_group: Option<BindGroup>,
    data_buffer: wgpu::Buffer,
    dim: (f32, f32),
    pos: (f32, f32),
    scale: f32,
}

impl App {
    // must be a multiple of 16 bytes
    fn buf_contents(&self) -> [u8; 32] {
        let mut ret = [0u8; 32];
        for (dst, src) in ret
            .chunks_exact_mut(4)
            .zip([self.dim.0, self.dim.1, self.pos.0, self.pos.1, self.scale])
        {
            dst.copy_from_slice(&src.to_le_bytes());
        }
        ret
    }
    fn load_shader(&mut self, shader: &str) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
            });
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&self.layout],
                push_constant_ranges: &[],
            });

        let swapchain_capabilities = self.surface.get_capabilities(&self.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        self.render_pipeline = Some(render_pipeline);
    }
    fn load_image(&mut self, img: image::DynamicImage) {
        let dimensions = img.dimensions();
        let rgba = img.into_rgba8();
        self.dim = (dimensions.0 as f32, dimensions.1 as f32);

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler1 = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler2 = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let buffer_binding = self.data_buffer.as_entire_buffer_binding();

        self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler1),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler2),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(buffer_binding),
                },
            ],
            label: None,
        }));
    }
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        mut platform: Platform,
    ) -> impl 'static + Future<Output = Self> {
        let window = event_loop
            .create_window(Platform::set_window_attrs(
                winit::window::WindowAttributes::default(),
            ))
            .unwrap();
        let mut size = window.inner_size();
        size.width = size.width.max(640);
        size.height = size.height.max(480);
        let instance = wgpu::Instance::default();
        platform.watch_file("nuero.png");
        platform.watch_file("shader.wgsl");

        async move {
            // XXX: I hate this, can't this be an Rc?
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
                        required_features: wgpu::Features::default(),
                        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                        required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                            .using_resolution(adapter.limits()),
                        memory_hints: wgpu::MemoryHints::MemoryUsage,
                    },
                    None,
                )
                .await
                .expect("Failed to create device");

            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });
            let data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: 32,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
            let reporter = platform.error_reporter();
            device.on_uncaptured_error(Box::new(move |error: wgpu::Error| {
                reporter(Box::new(error))
            }));

            let config = surface
                .get_default_config(&adapter, size.width, size.height)
                .unwrap();
            surface.configure(&device, &config);
            Self {
                config,
                device,
                adapter,
                queue,
                render_pipeline: None,
                surface,
                window,
                _platform: platform,
                bind_group: None,
                layout,
                data_buffer,
                dim: (0., 0.),
                pos: (0.0, 0.0),
                scale: 1.0,
            }
        }
    }
}

#[derive(Debug)]
enum Event {
    // Redraw,
    FileContents(String, Vec<u8>),
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.scale *= 1.1f32.powf(y);
                        // println!("scale1 {:?}", self.scale);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        self.scale *= 1.1f32.powf(delta.y as f32 * 0.1);
                        // println!("scale2 {:?}", self.scale);
                    }
                }
                self.window.request_redraw();
            }
            WindowEvent::PinchGesture { delta, .. } => {
                self.scale *= 1.1f32.powf(delta as f32);
                // println!("scale2 {:?}", self.scale);
                self.window.request_redraw();
            }
            WindowEvent::PanGesture { delta, .. } => {
                self.pos.0 += delta.x / 500. / self.scale;
                self.pos.1 += delta.y / 500. / self.scale;
                println!("pos {:?}", self.pos);
                self.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != winit::event::ElementState::Released {
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(c) => match c {
                            winit::keyboard::KeyCode::ArrowLeft => {
                                self.pos.0 += 0.1 / self.scale;
                                self.window.request_redraw();
                            }
                            winit::keyboard::KeyCode::ArrowRight => {
                                self.pos.0 -= 0.1 / self.scale;
                                self.window.request_redraw();
                            }
                            winit::keyboard::KeyCode::ArrowUp => {
                                self.pos.1 -= 0.1 / self.scale;
                                self.window.request_redraw();
                            }
                            winit::keyboard::KeyCode::ArrowDown => {
                                self.pos.1 += 0.1 / self.scale;
                                self.window.request_redraw();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                // Reconfigure the surface with the new size
                self.config.width = new_size.width.max(1);
                self.config.height = new_size.height.max(1);
                self.surface.configure(&self.device, &self.config);
                // On macos the window needs to be redrawn manually after resizing
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(pipeline), Some(group)) = (&self.render_pipeline, &self.bind_group) {
                    let frame = self
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    self.queue
                        .write_buffer(&self.data_buffer, 0, &self.buf_contents());
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, group, &[]);
                    rpass.draw(0..4, 0..1);
                    drop(rpass);
                    drop(view);
                    self.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        };
    }
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Event) {
        match event {
            // Event::Redraw => self.window.request_redraw(),
            Event::FileContents(name, contents) => match name.as_str() {
                "nuero.png" => {
                    if let Ok(img) = image::load_from_memory(&contents) {
                        self.load_image(img);
                        self.window.request_redraw();
                    }
                }
                "shader.wgsl" => {
                    if let Ok(code) = std::str::from_utf8(&contents) {
                        self.load_shader(code);
                        self.window.request_redraw();
                    }
                }
                _ => {}
            },
        }
    }
}

async fn run() {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    event_loop.run_app(&mut WinitProxy::Uninit(proxy)).unwrap();
}

pub fn start() {
    Platform::init();
    Platform::run_future(run());
}
