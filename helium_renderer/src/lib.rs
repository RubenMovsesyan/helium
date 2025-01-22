// std
use std::{sync::Arc, time::Instant};

use cgmath::{One, Quaternion, Vector3};
// Logging imports
use log::*;

use wgpu::InstanceDescriptor;
// winit imports
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Helium rendering modules
mod helium_state;
// Helium rendering imports
use helium_state::model::instance;
use helium_state::HeliumState;

pub async fn run() {
    info!("Starting window");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    _ = event_loop.run_app(&mut app);
}

// This is the actual window application that we will create
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<HeliumState>,

    // TEST: this is for testing time
    time: Option<Instant>,
}

// Implementation to handle the window application
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        self.time = Some(Instant::now());
        self.state = Some(HeliumState::new(self.window.as_ref().unwrap().clone()));
        // TEST: this is a test for object transformation
        self.state.as_mut().unwrap().create_instances(
            0,
            vec![instance::Instance::new(
                Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 1.0,
                },
                Quaternion::one(),
            )],
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let application = self.state.as_mut().unwrap();

        if self.window.as_ref().unwrap().id() == window_id {
            if application.input(&event) {
                return;
            }

            match event {
                WindowEvent::CloseRequested => {
                    info!("Window close requested; stopping");
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Redraw the application
                    if let Some(helium_state) = self.state.as_mut() {
                        helium_state.update_instance(
                            0,
                            instance::Instance {
                                position: Vector3 {
                                    x: f32::cos(
                                        (Instant::now() - *self.time.as_ref().unwrap())
                                            .as_secs_f32(),
                                    ),
                                    y: 0.0,
                                    z: f32::sin(
                                        (Instant::now() - *self.time.as_ref().unwrap())
                                            .as_secs_f32(),
                                    ),
                                },
                                rotation: Quaternion::one(),
                            },
                        );
                        helium_state.update();
                        helium_state.render().unwrap();
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Some(helium_state) = self.state.as_mut() {
                        helium_state.resize(new_size);
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
