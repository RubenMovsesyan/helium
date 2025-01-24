pub use cgmath::{One, Quaternion, Vector3, Zero};
pub use helium_compatibility::{Camera3d, Model3d, Transform3d};
pub use helium_ecs::Entity;
pub use helium_ecs::HeliumECS;
pub use helium_renderer::instance::Instance;
pub use helium_renderer::HeliumRenderer;
pub use helium_renderer::HeliumState;
use log::*;
pub use std::cell::{Ref, RefMut};
pub use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
pub use std::time::Instant;
pub use wgpu::SurfaceConfiguration;
pub use winit::event::{ElementState, KeyEvent, WindowEvent};
pub use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod helium_compatibility;

pub type StartupFunction = fn(&mut HeliumManager);
pub type UpdateFunction = fn(&mut HeliumManager);
pub type InputFunction = fn(&mut HeliumManager, &WindowEvent);

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

    pub fn move_camera_to_render(&mut self) {
        let camera_id = self.camera_id.as_ref().unwrap().clone();
        let cameras = self.query::<Camera3d>();
        let camera = cameras.get(&camera_id).unwrap().clone();
        drop(cameras);

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
    pub fn update_transform(&mut self, entity: Entity, transform: Transform3d) -> Entity {
        self.ecs_instance.add_component(entity, transform);

        // Get the renderer index from the model
        let object_index = self
            .ecs_instance
            .query::<Model3d>()
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
    pub fn query<ComponentType: 'static>(&mut self) -> Ref<'_, HashMap<Entity, ComponentType>> {
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
        &mut self,
    ) -> RefMut<'_, HashMap<Entity, ComponentType>> {
        self.ecs_instance.query_mut::<ComponentType>()
    }
}

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
    event_handler: Arc<Mutex<Option<WindowEvent>>>,
    /// Renderer for the window
    renderer: Option<Arc<Mutex<HeliumRenderer>>>,
    /// Thread that runs continuously to call update functions from the user
    update_thread: Option<thread::JoinHandle<()>>,
    /// Boolean to keep track of the running thread
    event_loop_working: Arc<Mutex<bool>>,
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
            event_handler: Arc::new(Mutex::new(None)),
            renderer: None,
            update_thread: None,
            event_loop_working: Arc::new(Mutex::new(false)),
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

        // This is the main update loop of the game
        let startup_functions_clone = self.startup_functions.clone();
        let update_functions_clone = self.update_functions.clone();
        let input_functions_clone = self.input_functions.clone();
        let renderer_clone = self.renderer.as_ref().unwrap().clone();
        let event_handler_clone = self.event_handler.clone();

        // For making sure this thread ends as soon as the main thread ends
        let event_loop_working_clone = self.event_loop_working.clone();
        self.update_thread = Some(thread::spawn(move || {
            let new_ecs = HeliumECS::new();
            let mut manager = HeliumManager::new(new_ecs, renderer_clone);
            info!("Starting Helium ECS");

            for startup_function in startup_functions_clone.lock().as_ref().unwrap().iter() {
                startup_function(&mut manager);
            }
            info!("Starup functions complete, Running Updates");

            loop {
                for update_function in update_functions_clone.lock().as_ref().unwrap().iter() {
                    update_function(&mut manager);
                }

                // Handle any necessary window events here
                if let Some(event) = event_handler_clone.lock().unwrap().take() {
                    for input_function in input_functions_clone.lock().unwrap().iter() {
                        input_function(&mut manager, &event);
                    }
                }
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
            // if self
            //     .renderer
            //     .as_ref()
            //     .unwrap()
            //     .clone()
            //     .lock()
            //     .as_mut()
            //     .unwrap()
            //     .state
            //     .input(&event)
            // {
            //     return;
            // }

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
                        // renderer.update();
                        renderer.render();
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Ok(renderer) = self.renderer.as_ref().unwrap().clone().lock().as_mut() {
                        renderer.resize(new_size);
                    }
                }
                _ => {}
            }

            self.event_handler.lock().unwrap().replace(event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
