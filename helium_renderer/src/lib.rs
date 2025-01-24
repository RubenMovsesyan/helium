// std
use std::{sync::Arc, time::Instant};

// use cgmath::{One, Quaternion, Vector3};
// Logging imports

use wgpu::{Device, Queue};
// winit imports
use winit::{dpi::PhysicalSize, window::Window};

// Helium rendering modules
pub mod helium_state;
// Helium rendering imports
pub use helium_state::model::instance;
pub use helium_state::model::Model;
pub use helium_state::Camera;
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
