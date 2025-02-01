use cgmath::InnerSpace;
pub use cgmath::Point3;
use helium_compatibility::CAMERA_SPEED;
// logging
use log::*;

// std imports
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// std imports to be broadcast
pub use std::cell::{Ref, RefMut};
pub use std::collections::HashMap;
pub use std::time::Instant;

// wgpu imports
pub use wgpu::SurfaceConfiguration;

// Math
pub use cgmath::{One, Quaternion, Vector3, Zero};

// Winit imports
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// Helium compatibility imports
pub use helium_collisions::collider::{Collider, RectangleCollider, StationaryPlaneCollider};
pub use helium_compatibility::{Camera3d, CameraController, Label, Model3d, Transform3d};
pub use helium_ecs::{Entity, HeliumECS};
pub use helium_manager::HeliumManager;
pub use helium_physics::gravity::Gravity;
pub use helium_renderer::{instance::Instance, HeliumState, Light};

mod helium_compatibility;
mod helium_manager;
// Custom type aliases for simplicity
pub type InputEvent = DeviceEvent;
pub type StartupFunction = fn(&mut HeliumManager);
pub type UpdateFunction = fn(&mut HeliumManager);
pub type InputFunction = fn(&mut HeliumManager, &InputEvent);

// Internal function for handling collisions if they are turned on
fn handle_gravity_collisions(manager: &mut HeliumManager) {
    let stationary_plane_colliders = match manager.query::<StationaryPlaneCollider>() {
        Some(plane_colliders) => plane_colliders,
        None => return,
    };

    let mut rectangle_colliders = match manager.query_mut::<RectangleCollider>() {
        Some(rectangle_colliders) => rectangle_colliders,
        None => return,
    };

    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    let mut gravities = match manager.query_mut::<Gravity>() {
        Some(gravities) => gravities,
        None => return,
    };

    for (entity, rectangle_colider) in rectangle_colliders.iter_mut() {
        if let Some(gravity) = gravities.get_mut(entity) {
            gravity.update_gravity(&manager.delta_time);

            if let Some(transform) = transforms.get_mut(entity) {
                for (_, plane_collider) in stationary_plane_colliders.iter() {
                    if rectangle_colider.is_colliding(plane_collider) {
                        rectangle_colider.snap_y(plane_collider);
                        gravity.kill_velocity();
                    }
                }

                transform
                    .add_position(gravity.velocity * manager.delta_time.elapsed().as_secs_f32());
            }
        }
    }
}

fn update_cameras(manager: &mut HeliumManager) {
    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    let mut cameras = match manager.query_mut::<Camera3d>() {
        Some(cameras) => cameras,
        None => return,
    };

    let mut camera_controllers = match manager.query_mut::<CameraController>() {
        Some(controllers) => controllers,
        None => return,
    };

    // If any of the above doesn't exist there is no point of continuing on

    for (entity, controller) in camera_controllers.iter_mut() {
        if let Some(camera) = cameras.get_mut(entity) {
            camera.add_yaw(-controller.delta.0);
            camera.add_pitch(-controller.delta.1);
            controller.delta = (0.0, 0.0);

            if let Some(transform) = transforms.get_mut(entity) {
                let forward_norm = camera.target.normalize();

                if controller.forward {
                    transform.add_position(
                        forward_norm * manager.delta_time.elapsed().as_secs_f32() * CAMERA_SPEED,
                    );
                }

                if controller.backward {
                    transform.add_position(
                        -forward_norm * manager.delta_time.elapsed().as_secs_f32() * CAMERA_SPEED,
                    );
                }

                let right = forward_norm.cross(camera.up);

                if controller.left {
                    transform.add_position(
                        -right * manager.delta_time.elapsed().as_secs_f32() * CAMERA_SPEED,
                    );
                }

                if controller.right {
                    transform.add_position(
                        right * manager.delta_time.elapsed().as_secs_f32() * CAMERA_SPEED,
                    );
                }
            }

            manager.move_camera_to_render(camera);
        }
    }
}

fn update_transforms_to_renderer(manager: &mut HeliumManager) {
    // List of transforms to look through and update
    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    // Models to update in the renderer
    let models = manager.query::<Model3d>();

    // Camera to update if it exists
    let mut cameras = manager.query_mut::<Camera3d>();

    // Rectangle Colliders to update if it exists
    let mut colliders = manager.query_mut::<RectangleCollider>();

    // Lights to update if exists
    let mut lights = manager.query_mut::<Light>();

    for (entity, transform) in transforms.iter_mut() {
        if !transform.get_update_flag() {
            continue;
        }

        // Update the model position
        if let Some(models) = models.as_ref() {
            if let Some(object_index) = models.get(entity) {
                manager.renderer_instance.lock().unwrap().update_instances(
                    *object_index.get_renderer_index().unwrap(),
                    vec![(*transform).into()],
                );
            }
        }

        // Update the Camera position
        if let Some(cameras) = cameras.as_mut() {
            if let Some(camera) = cameras.get_mut(entity) {
                let pos = transform.get_position();
                camera.set_position(cgmath::point3::<f32>(pos.x, pos.y, pos.z));
            }
        }

        // Update the colliders position
        if let Some(colliders) = colliders.as_mut() {
            if let Some(collider) = colliders.get_mut(entity) {
                collider.set_origin(transform.get_position());
            }
        }

        // Update the lights position
        if let Some(lights) = lights.as_mut() {
            if let Some(light) = lights.get_mut(entity) {
                light.update_position(transform.get_position());
                manager
                    .renderer_instance
                    .lock()
                    .unwrap()
                    .update_light(light);
            }
        }

        transform.update();
    }
}

// Helium instance

pub struct Helium {
    /// This is the Helium window that opens
    event_loop: Option<EventLoop<()>>,
    /// These functions will run when the window
    startup_functions: Arc<Mutex<Vec<StartupFunction>>>,
    /// These functions will run whenever and update is requested
    update_functions: Arc<Mutex<Vec<UpdateFunction>>>,
    /// These functions will run whenever the input is called
    input_functions: Arc<Mutex<Vec<InputFunction>>>,
    /// Winit instance
    window: Option<Arc<Window>>,
    /// Event handling for the window
    event_handler: Arc<Mutex<VecDeque<InputEvent>>>,
    /// Renderer for the window
    renderer: Option<Arc<Mutex<HeliumState>>>,
    /// Thread that runs continuously to call update functions from the user
    update_thread: Option<thread::JoinHandle<()>>,
    /// Boolean to keep track of the running thread
    event_loop_working: Arc<Mutex<bool>>,
    /// Time to keep track of fps
    fps: Instant,
}

impl Default for Helium {
    fn default() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        Self {
            event_loop: Some(event_loop),
            startup_functions: Arc::new(Mutex::new(Vec::new())),
            update_functions: Arc::new(Mutex::new(Vec::new())),
            input_functions: Arc::new(Mutex::new(Vec::new())),
            window: None,
            event_handler: Arc::new(Mutex::new(VecDeque::new())),
            renderer: None,
            update_thread: None,
            event_loop_working: Arc::new(Mutex::new(false)),
            fps: Instant::now(),
        }
    }
}

impl Helium {
    /// Adds a startup function to be executed when the engine starts
    ///
    /// # Arguments
    ///
    /// * `startup_function` - Function pointer to run at startup
    ///
    /// # Returns
    ///
    /// A mutable reference to self
    pub fn add_startup(&mut self, startup_function: StartupFunction) -> &mut Self {
        self.startup_functions
            .lock()
            .as_mut()
            .unwrap()
            .push(startup_function);
        self
    }

    /// Adds an input function to be executed when the input handler is called
    ///
    /// # Arguments
    ///
    /// * `input_function` - Function pointer to run on input
    ///
    /// # Returns
    ///
    /// A mutable reference to self
    pub fn add_input(&mut self, input_function: InputFunction) -> &mut Self {
        self.input_functions
            .lock()
            .as_mut()
            .unwrap()
            .push(input_function);
        self
    }

    /// Adds an update function to be executed while the engine is running
    ///
    /// # Arguments
    ///
    /// * `update_function` - Function pointer to run continuously
    ///
    /// # Returns
    ///
    /// A mutable reference to self
    pub fn add_update(&mut self, update_function: UpdateFunction) -> &mut Self {
        self.update_functions
            .lock()
            .as_mut()
            .unwrap()
            .push(update_function);
        self
    }

    pub fn run(&mut self) {
        pretty_env_logger::init();
        info!("Starting Helium Window");

        *self.event_loop_working.lock().unwrap() = true;
        _ = self.event_loop.take().unwrap().run_app(self);
    }
}

impl ApplicationHandler for Helium {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        // self.renderer = Some(Arc::new(Mutex::new(HeliumRenderer::new(
        //     self.window.as_ref().unwrap().clone(),
        // ))));
        self.renderer = Some(Arc::new(Mutex::new(HeliumState::new(
            self.window.as_ref().unwrap().clone(),
        ))));

        // Create arc clones to pass to the ecs
        let startup_functions_clone = self.startup_functions.clone();
        let update_functions_clone = self.update_functions.clone();
        let input_functions_clone = self.input_functions.clone();
        let renderer_clone = self.renderer.as_ref().unwrap().clone();
        let event_handler_clone = self.event_handler.clone();

        // For making sure this thread ends as soon as the main thread ends
        let event_loop_working_clone = self.event_loop_working.clone();

        // This is the continuously running update thread
        self.update_thread = Some(thread::spawn(move || {
            let new_ecs = HeliumECS::default();
            let mut manager = HeliumManager::new(new_ecs, renderer_clone);
            info!("Starting Helium ECS");

            // Run all the starup functions when starting the update thread
            for startup_function in startup_functions_clone.lock().as_ref().unwrap().iter() {
                startup_function(&mut manager);
            }
            info!("Starup functions complete, Running Updates");

            loop {
                // Handle all updates
                for update_function in update_functions_clone.lock().as_ref().unwrap().iter() {
                    update_function(&mut manager);
                }

                // Handle any necessary window events here
                while let Some(event) = event_handler_clone.lock().unwrap().pop_front() {
                    for input_function in input_functions_clone.lock().unwrap().iter() {
                        input_function(&mut manager, &event);
                    }
                }

                // Handle collisions
                handle_gravity_collisions(&mut manager);
                // Update all the changed transforms
                update_transforms_to_renderer(&mut manager);
                // Handle cameras
                update_cameras(&mut manager);
                // Handle lights
                manager.delta_time = Instant::now();

                if !(*event_loop_working_clone.lock().unwrap()) {
                    break;
                }
            }
        }));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.window.as_ref().unwrap().id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Window close requested; stopping");
                    *self.event_loop_working.lock().unwrap() = false;
                    self.update_thread.take().unwrap().join().unwrap();
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Redraw the application
                    if let Ok(renderer) = self.renderer.as_ref().unwrap().clone().lock().as_mut() {
                        renderer.fps =
                            format!("{:>7.2} FPS", 1.0 / self.fps.elapsed().as_secs_f32());
                        _ = renderer.render();
                        self.fps = Instant::now();
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Ok(renderer) = self.renderer.as_ref().unwrap().clone().lock().as_mut() {
                        renderer.resize(new_size);
                        renderer.brush.resize_view(
                            renderer.config.width as f32,
                            renderer.config.height as f32,
                            renderer.get_queue(),
                        )
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.event_handler.lock().unwrap().push_back(event);
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
