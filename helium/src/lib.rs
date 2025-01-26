use helium_compatibility::transform;
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
pub use helium_physics::gravity::Gravity;
pub use helium_renderer::{instance::Instance, HeliumRenderer, HeliumState};
mod helium_compatibility;

// Custom type aliases for simplicity
pub type InputEvent = DeviceEvent;
pub type StartupFunction = fn(&mut HeliumManager);
pub type UpdateFunction = fn(&mut HeliumManager);
pub type InputFunction = fn(&mut HeliumManager, &InputEvent);

pub struct HeliumManager {
    ecs_instance: HeliumECS,
    renderer_instance: Arc<Mutex<HeliumRenderer>>,

    // For easy access to the camera
    pub camera_id: Option<Entity>,

    pub time: Instant,
    pub delta_time: Instant,
}

impl HeliumManager {
    fn new(ecs: HeliumECS, renderer: Arc<Mutex<HeliumRenderer>>) -> Self {
        Self {
            ecs_instance: ecs,
            renderer_instance: renderer.clone(),
            camera_id: None,
            time: Instant::now(),
            delta_time: Instant::now(),
        }
    }

    pub fn get_render_config(&self) -> SurfaceConfiguration {
        self.renderer_instance.lock().unwrap().state.config.clone()
    }

    /// Creates a 3d camera to view the scene with. The rendering will be skipped if
    /// No cameara is present
    ///
    /// # Arguments
    ///
    /// * `camera` - The `Camera3d` that will be added to the scene
    ///
    /// # Returns
    ///
    /// The entity id
    pub fn create_camera(&mut self, camera: Camera3d) -> Entity {
        self.renderer_instance.lock().unwrap().state.add_camera(
            camera.eye,
            camera.target,
            camera.up,
            camera.aspect,
            camera.fovy,
            camera.znear,
            camera.zfar,
        );

        let camera_entity = self.ecs_instance.new_entity();
        self.ecs_instance.add_component(camera_entity, camera);
        self.ecs_instance.add_component(
            camera_entity,
            CameraController {
                forward: false,
                backward: false,
                left: false,
                right: false,
                delta: (0.0, 0.0),
            },
        );
        self.camera_id = Some(camera_entity);
        camera_entity
    }

    /// Updates the camera based on the new camera provided
    ///
    /// # Arguments
    ///
    /// * `camera` - the new camera
    pub fn update_camera(&mut self, camera: Camera3d) {
        self.renderer_instance.lock().unwrap().state.update_camera(
            camera.eye,
            camera.target,
            camera.up,
            camera.aspect,
            camera.fovy,
            camera.znear,
            camera.zfar,
        );
        self.ecs_instance
            .add_component(*self.camera_id.as_ref().unwrap(), camera);
    }

    /// Used internally to update the camera position
    pub fn move_camera_to_render(&self, camera: &Camera3d) {
        self.renderer_instance.lock().unwrap().state.update_camera(
            camera.eye,
            camera.target,
            camera.up,
            camera.aspect,
            camera.fovy,
            camera.znear,
            camera.zfar,
        );
    }

    /// Creates a new entity in the ECS
    ///
    /// # Returns
    ///
    /// An `Entity` id
    pub fn create_entity(&mut self) -> Entity {
        self.ecs_instance.new_entity()
    }

    /// Creates a 3d model component with the required transform component
    ///
    /// # Arguments
    ///
    /// * `model` - The 3d model to import into the engin
    /// * `transform` - The transformation to apply to the model
    ///
    /// # Returns
    ///
    /// The entity id
    pub fn create_object(&mut self, mut model: Model3d, transform: Transform3d) -> Entity {
        let renderer_index = self
            .renderer_instance
            .lock()
            .unwrap()
            .state
            .create_object(model.get_path(), vec![transform.clone().into()]);

        model.set_renderer_index(renderer_index);

        // let mut ecs = self.ecs_instance;
        let entity = self.ecs_instance.new_entity();
        self.ecs_instance.add_component(entity, model);
        self.ecs_instance.add_component(entity, transform);

        entity
    }

    /// Sets the transform for a specified entity to a new transform
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to change the transform of
    /// * `transform` - The new Transform to set for the entity
    ///
    /// # Returns
    ///
    /// The entity id
    #[deprecated]
    pub fn update_transform(&mut self, entity: Entity, transform: Transform3d) -> Entity {
        self.ecs_instance.add_component(entity, transform);

        // Get the renderer index from the model
        let object_index = self
            .ecs_instance
            .query::<Model3d>()
            .unwrap()
            .get(&entity)
            .unwrap()
            .get_renderer_index()
            .unwrap()
            .clone();

        let mut renderer = self.renderer_instance.lock().unwrap();
        renderer
            .state
            .update_instances(object_index, vec![transform.into()]);

        entity
    }

    #[deprecated]
    pub fn set_position(&mut self, entity: Entity, position: Vector3<f32>) {
        let object_index = self
            .ecs_instance
            .query::<Model3d>()
            .unwrap()
            .get(&entity)
            .unwrap()
            .get_renderer_index()
            .unwrap()
            .clone();
        if let Some(transform) = self
            .ecs_instance
            .query_mut::<Transform3d>()
            .unwrap()
            .get_mut(&entity)
        {
            Transform3d::set_position(transform, position);
            self.renderer_instance
                .lock()
                .unwrap()
                .state
                .update_instances(object_index, vec![transform.clone().into()]);
        }
    }

    #[deprecated]
    pub fn set_rotation(&mut self, entity: Entity, rotation: Quaternion<f32>) {
        let object_index = self
            .ecs_instance
            .query::<Model3d>()
            .unwrap()
            .get(&entity)
            .unwrap()
            .get_renderer_index()
            .unwrap()
            .clone();
        if let Some(transform) = self
            .ecs_instance
            .query_mut::<Transform3d>()
            .unwrap()
            .get_mut(&entity)
        {
            Transform3d::set_rotation(transform, rotation);
            self.renderer_instance
                .lock()
                .unwrap()
                .state
                .update_instances(object_index, vec![transform.clone().into()]);
        }
    }

    #[deprecated]
    pub fn move_transform_to_renderer(&self, entity: Entity) {
        let object_index = self
            .ecs_instance
            .query::<Model3d>()
            .unwrap()
            .get(&entity)
            .unwrap()
            .get_renderer_index()
            .unwrap()
            .clone();
        let transforms = self.ecs_instance.query::<Transform3d>();
        if let Some(transform) = transforms.unwrap().get(&entity) {
            self.renderer_instance
                .lock()
                .unwrap()
                .state
                .update_instances(object_index, vec![transform.clone().into()]);
        }
    }

    /// Adds a component to the specified entity
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to add componenets to
    /// * `component` - Component to add to the entity
    ///
    /// # Returns
    ///
    /// The entity id
    pub fn add_component<ComponentType: 'static>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) -> Entity {
        self.ecs_instance.add_component(entity, component);
        entity
    }

    /// Querys the ECS for the component type specified and gives the corresponding information
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the ECS to query for
    ///
    /// # Returns
    ///
    /// A `Ref` to the `HashMap` of the specified `ComponentType`
    pub fn query<ComponentType: 'static>(&self) -> Option<Ref<'_, HashMap<Entity, ComponentType>>> {
        self.ecs_instance.query::<ComponentType>()
    }

    /// Querys the ECS for the component type specified and gives the corresponding information
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the ECS to query for
    ///
    /// # Returns
    ///
    /// A `RefMut` to the `HashMap` of the specified `ComponentType`
    pub fn query_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<'_, HashMap<Entity, ComponentType>>> {
        self.ecs_instance.query_mut::<ComponentType>()
    }

    /// Gives a list of entities that have a component with a specific comparator operator
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component map to seach
    /// * `comparator` - A fucntion pointer to compare the component value given
    ///
    /// # Returns
    ///
    /// A list of entities that contain the specified property
    pub fn entities_with<ComponentType: 'static>(
        &self,
        comparator: fn(&ComponentType) -> bool,
    ) -> Vec<Entity> {
        self.ecs_instance.entities_with::<ComponentType>(comparator)
    }
}

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
        for (_, plane_collider) in stationary_plane_colliders.iter() {
            if rectangle_colider.is_colliding_y(plane_collider) {
                // info!("Colliding! {:#?} {:#?}", rectangle_colider, plane_collider);
                rectangle_colider.snap_y(plane_collider);

                if let Some(gravity) = gravities.get_mut(entity) {
                    gravity.kill_velocity();
                }

                if let Some(transform) = transforms.get_mut(&entity) {
                    transform.update_position(*rectangle_colider.origin());
                }
            }
        }

        if let Some(transform) = transforms.get(&entity) {
            rectangle_colider.origin = transform.position;
        }
    }
}

fn handle_gravity(manager: &mut HeliumManager) {
    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    let mut gravities = match manager.query_mut::<Gravity>() {
        Some(gravities) => gravities,
        None => return,
    };

    let mut entity_update_list = Vec::new();
    for (entity, gravity) in gravities.iter_mut() {
        gravity.update_gravity(&manager.delta_time);
        if let Some(transform) = transforms.get_mut(entity) {
            transform.add_position(gravity.velocity * manager.delta_time.elapsed().as_secs_f32());
            entity_update_list.push(*entity);
        }
    }
}

fn update_transforms_to_renderer(manager: &mut HeliumManager) {
    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    let models = match manager.query::<Model3d>() {
        Some(models) => models,
        None => return,
    };

    for (entity, transform) in transforms.iter_mut() {
        if !transform.update_flag {
            continue;
        }
        let object_index = *models
            .get(&entity)
            .unwrap()
            .get_renderer_index()
            .clone()
            .unwrap();

        manager
            .renderer_instance
            .lock()
            .unwrap()
            .state
            .update_instances(object_index, vec![transform.clone().into()]);

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
    renderer: Option<Arc<Mutex<HeliumRenderer>>>,
    /// Thread that runs continuously to call update functions from the user
    update_thread: Option<thread::JoinHandle<()>>,
    /// Boolean to keep track of the running thread
    event_loop_working: Arc<Mutex<bool>>,
    /// Time to keep track of fps
    fps: Instant,
}

impl Helium {
    pub fn new() -> Self {
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

        self.renderer = Some(Arc::new(Mutex::new(HeliumRenderer::new(
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
            let new_ecs = HeliumECS::new();
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

                // HACK: handle the camera update here
                // This can probably be done in a better place
                // let mut camera_controllers = manager.query_mut::<CameraController>();
                // let mut cameras = manager.query_mut::<Camera3d>();
                if let Some(mut camera_controllers) = manager.query_mut::<CameraController>() {
                    if let Some(mut cameras) = manager.query_mut::<Camera3d>() {
                        let cam_and_controllers = cameras
                            .iter_mut()
                            .zip(camera_controllers.iter_mut())
                            .filter_map(|(camera, controller)| Some((camera.1, controller.1)));

                        for (camera, controller) in cam_and_controllers {
                            camera.update_camera(
                                controller.forward,
                                controller.backward,
                                controller.left,
                                controller.right,
                                &manager.delta_time,
                            );
                            camera.add_yaw(-controller.delta.0);
                            camera.add_pitch(-controller.delta.1);
                            controller.delta = (0.0, 0.0);
                            manager.move_camera_to_render(camera);
                        }
                    }
                }

                // Handle collisions
                handle_gravity_collisions(&mut manager);

                // Handle Gravity
                handle_gravity(&mut manager);

                // Update all the changed transforms
                update_transforms_to_renderer(&mut manager);

                manager.delta_time = Instant::now();

                if *event_loop_working_clone.lock().unwrap() == false {
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
                        renderer.state.fps =
                            format!("{:>7.2} FPS", 1.0 / self.fps.elapsed().as_secs_f32());
                        renderer.render();
                        self.fps = Instant::now();
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Ok(renderer) = self.renderer.as_ref().unwrap().clone().lock().as_mut() {
                        renderer.resize(new_size);
                        renderer.state.brush.resize_view(
                            renderer.state.config.width as f32,
                            renderer.state.config.height as f32,
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
