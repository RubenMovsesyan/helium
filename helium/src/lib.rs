pub use cgmath::{One, Quaternion, Vector3, Zero};
pub use helium_renderer::instance::Instance;
use helium_renderer::HeliumRenderer;
pub use helium_renderer::HeliumState;
use helium_renderer::{StartupFunction, UpdateFunction};
use smol::block_on;
pub use std::time::Instant;

pub struct Helium {
    renderer: HeliumRenderer,
}

impl Helium {
    pub fn new() -> Self {
        Self {
            renderer: HeliumRenderer::new(),
        }
    }

    pub fn add_startup(&mut self, startup_function: StartupFunction) -> &mut Self {
        self.renderer.add_startup(startup_function);
        self
    }

    pub fn add_update(&mut self, update_function: UpdateFunction) -> &mut Self {
        self.renderer.add_update(update_function);
        self
    }

    pub fn run(&mut self) {
        pretty_env_logger::init();
        block_on(self.renderer.run());
    }
}
