//! A proxy that initializes the "real" app on first `resumed` call and then transparently routes all
//! calls to it
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoopProxy,
    window::WindowId,
};

use crate::{platform::PlatformTrait, run_future, App, Event};

#[derive(Debug)]
pub enum WinitProxy {
    Uninit(EventLoopProxy<ProxyEvent>),
    Waiting {
        window_events: Vec<(WindowId, WindowEvent)>,
        user_events: Vec<Event>,
    },
    Init(App),
}

#[derive(Debug)]
pub enum ProxyEvent {
    Init(App),
    Event(Event),
}

#[derive(Clone, Debug)]
pub struct SendEvent(EventLoopProxy<ProxyEvent>);
impl SendEvent {
    pub fn send_event(&self, event: crate::Event) {
        self.0
            .send_event(ProxyEvent::Event(event))
            .map_err(|_| "event loop closed")
            .unwrap();
    }
}

impl ApplicationHandler<ProxyEvent> for WinitProxy {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            Self::Uninit(_) => {
                let mut tmp = Self::Waiting {
                    window_events: Vec::new(),
                    user_events: Vec::new(),
                };
                std::mem::swap(&mut tmp, self);
                let Self::Uninit(proxy) = tmp else {
                    unreachable!()
                };
                let fut = App::new(event_loop, crate::Platform::new(SendEvent(proxy.clone())));
                run_future(async move {
                    proxy
                        .send_event(ProxyEvent::Init(fut.await))
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
            Self::Waiting { window_events, .. } => window_events.push((window_id, event)),
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
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ProxyEvent) {
        match event {
            ProxyEvent::Init(mut app) => {
                app.resumed(event_loop);
                let mut this = Self::Init(app);
                std::mem::swap(&mut this, self);
                let Self::Waiting {
                    window_events,
                    user_events,
                } = this
                else {
                    return;
                };
                for (window_id, event) in window_events {
                    self.window_event(event_loop, window_id, event)
                }
                for event in user_events {
                    self.user_event(event_loop, ProxyEvent::Event(event))
                }
            }
            ProxyEvent::Event(event) => match self {
                Self::Init(this) => this.user_event(event_loop, event),
                Self::Waiting { user_events, .. } => user_events.push(event),
                _ => {}
            },
        }
    }
}
