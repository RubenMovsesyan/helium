// std
use std::{sync::Arc, time::Instant};

// use cgmath::{One, Quaternion, Vector3};
// Logging imports
use log::*;

use wgpu::{Device, Queue};
// winit imports
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

// Helium rendering modules
pub mod helium_state;
// Helium rendering imports
pub use helium_state::model::instance;
pub use helium_state::model::Model;
pub use helium_state::HeliumState;

pub type StartupFunction = fn(&mut HeliumState);
pub type UpdateFunction = fn(&mut HeliumState, Instant);

pub struct HeliumRenderer {
    pub state: HeliumState,
    startup_functions: Vec<StartupFunction>,
    update_functions: Vec<UpdateFunction>,
}

impl HeliumRenderer {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            state: HeliumState::new(window.clone()),
            startup_functions: Vec::new(),
            update_functions: Vec::new(),
        }
    }

    pub fn set_startup(&mut self, startup_functions: Vec<StartupFunction>) {
        self.startup_functions = startup_functions;
    }

    pub fn set_update(&mut self, update_functions: Vec<UpdateFunction>) {
        self.update_functions = update_functions;
    }

    pub fn update(&mut self) {
        self.state.update();
    }

    pub fn render(&mut self) {
        _ = self.state.render();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.state.resize(new_size);
    }

    pub fn get_device(&self) -> &Device {
        self.state.get_device()
    }

    pub fn get_queue(&self) -> &Queue {
        self.state.get_queue()
    }
}

// This is the actual window application that we will create
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    state: Option<HeliumState>,
    startup_fn: Vec<StartupFunction>,
    update_fn: Vec<UpdateFunction>,
    /// This is for keeping track of time between updates
    time: Option<Instant>,
}

impl App {
    pub fn set_startup(&mut self, startup_fn: Vec<StartupFunction>) {
        self.startup_fn = startup_fn;
    }

    pub fn set_update(&mut self, update_fn: Vec<UpdateFunction>) {
        self.update_fn = update_fn;
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

        // This is where all the startup functions get run
        for startup_function in self.startup_fn.iter() {
            startup_function(self.state.as_mut().unwrap());
        }
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
                        // Run all the update fuctions per frame here
                        for update_function in self.update_fn.iter() {
                            update_function(helium_state, *self.time.as_ref().unwrap());
                        }
                        helium_state.update();
                        helium_state.render().unwrap();
                    }
                    self.time = Some(Instant::now());
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
