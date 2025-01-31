use crate::helium_compatibility::{Camera3d, Model3d, Transform3d};
pub use cgmath::{Quaternion, Vector3};
pub use helium_ecs::{Entity, HeliumECS};
use helium_renderer::{HeliumRenderer, Light};
pub use std::cell::{Ref, RefMut};
pub use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wgpu::SurfaceConfiguration;

pub struct HeliumManager {
    pub ecs_instance: HeliumECS,
    pub renderer_instance: Arc<Mutex<HeliumRenderer>>,

    // For easy access to the camera
    pub camera_id: Option<Entity>,

    pub time: Instant,
    pub delta_time: Instant,
}

impl HeliumManager {
    pub fn new(ecs: HeliumECS, renderer: Arc<Mutex<HeliumRenderer>>) -> Self {
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

    pub fn add_light(&mut self, mut light: Light) -> Entity {
        self.renderer_instance
            .lock()
            .unwrap()
            .state
            .add_light(&mut light);

        let light_entity = self.ecs_instance.new_entity();
        self.ecs_instance.add_component(light_entity, light.clone());
        light_entity
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
        // self.ecs_instance.add_component(
        //     camera_entity,
        //     CameraController {
        //         forward: false,
        //         backward: false,
        //         left: false,
        //         right: false,
        //         delta: (0.0, 0.0),
        //     },
        // );
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

    // #[deprecated]
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
