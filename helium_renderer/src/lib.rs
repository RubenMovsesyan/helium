// std
use std::{sync::Arc, time::Instant};

use cgmath::{One, Quaternion, Vector3};
// Logging imports
use log::*;

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
pub use helium_state::model::instance;
pub use helium_state::HeliumState;

pub type StartupFunction = fn(&mut HeliumState);

pub struct HeliumRenderer {
    event_loop: Option<EventLoop<()>>,
    startup_fn: Option<Vec<StartupFunction>>,
}

impl HeliumRenderer {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        Self {
            event_loop: Some(event_loop),
            startup_fn: None,
        }
    }

    pub fn add_startup(&mut self, startup_function: StartupFunction) -> &mut Self {
        if let Some(startup_fn_vec) = self.startup_fn.as_mut() {
            startup_fn_vec.push(startup_function);
        } else {
            self.startup_fn = Some(vec![startup_function]);
        }

        self
    }

    pub async fn run(&mut self) {
        info!("Starting window");
        let mut app = App::default();

        if let Some(startup_functions) = self.startup_fn.take() {
            app.set_startup(startup_functions);
        }

        _ = self.event_loop.take().unwrap().run_app(&mut app);
    }
}

// This is the actual window application that we will create
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<HeliumState>,
    startup_fn: Vec<StartupFunction>,
    // TEST: this is for testing time
    time: Option<Instant>,
}

impl App {
    fn set_startup(&mut self, startup_fn: Vec<StartupFunction>) -> &mut Self {
        self.startup_fn = startup_fn;
        self
    }
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
        for startup_function in self.startup_fn.iter() {
            startup_function(self.state.as_mut().unwrap());
        }
        // self.state
        //     .as_mut()
        //     .unwrap()
        //     .create_object("./assets/suzzane.obj", {
        //         let mut instances = Vec::new();
        //         for i in 0..1 {
        //             instances.push(instance::Instance {
        //                 position: Vector3 {
        //                     x: 1.0 * i as f32,
        //                     y: 0.0,
        //                     z: 0.0,
        //                 },
        //                 rotation: Quaternion::one(),
        //             });
        //         }
        //         instances
        //     });
        // self.state.as_mut().unwrap().create_instances(0, {
        //     let mut instances = Vec::new();
        //     for i in 0..1 {
        //         instances.push(instance::Instance {
        //             position: Vector3 {
        //                 x: 1.0 * i as f32,
        //                 y: 0.0,
        //                 z: 0.0,
        //             },
        //             rotation: Quaternion::one(),
        //         });
        //     }
        //     instances
        // });
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
                        helium_state.update_instances(0, {
                            let mut instances = Vec::new();

                            for i in -5..=5 {
                                instances.push(instance::Instance {
                                    position: Vector3 {
                                        x: 5.0 * i as f32,
                                        y: 1.0
                                            * f32::sin(
                                                (Instant::now() - *self.time.as_ref().unwrap())
                                                    .as_secs_f32()
                                                    + i as f32,
                                            ),
                                        z: 1.0
                                            * f32::cos(
                                                (Instant::now() - *self.time.as_ref().unwrap())
                                                    .as_secs_f32()
                                                    - i as f32,
                                            ),
                                    },
                                    rotation: Quaternion::one(),
                                })
                            }

                            instances
                        });
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
