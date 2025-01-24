// std
use cgmath::Point3;
use cgmath::Vector3;
use std::{iter::once, path::Path, sync::Arc};
// Async
use smol::block_on;

// wgpu imports
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Backends, Buffer, BufferUsages, Color, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations,
    PowerPreference, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RequestAdapterOptionsBase, StoreOp, Surface, SurfaceCapabilities, SurfaceConfiguration,
    SurfaceError, TextureUsages, TextureViewDescriptor,
};

// winit imports
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

// logging
use log::*;

// State handling modules
pub mod camera;
mod helium_texture;
pub mod model;
mod resources;

// module imports
pub use camera::Camera;
use helium_texture::HeliumTexture;
use model::{
    instance::INSTANCE_RAW_SIZE, model_vertex::ModelVertex, render_pipeline::HeliumRenderPipeline,
    Model,
};

pub use model::instance;

pub struct HeliumState {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    pub config: SurfaceConfiguration,

    // wgpu rendering varables

    // Camera
    camera: Camera,
    camera_active: bool,
    // camera_controller: CameraController,

    // Depth texture for rendering the correct faces of a mesh
    depth_texture: HeliumTexture,

    // current pipeline for rendering
    render_pipeline: Arc<RenderPipeline>,

    // Models to render
    models: Vec<Model>,

    // Instances for all the instance
    model_instances: Vec<instance::Instance>,

    // Instance buffer for all the instances
    model_instance_buffer: Buffer,
}

impl HeliumState {
    // Set the instances for a particular object in the state
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }

    pub fn create_instances(
        &mut self,
        object_index: usize,
        mut instances: Vec<instance::Instance>,
    ) {
        let range_start = self.model_instances.len() as u32;
        self.model_instances.append(&mut instances);
        let range_end = self.model_instances.len() as u32;

        self.models[object_index].set_instances(range_start..range_end);

        self.model_instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model instance buffer"),
            contents: bytemuck::cast_slice(
                self.model_instances
                    .iter()
                    .map(|instance| instance.to_raw())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        self.queue.write_buffer(
            &self.model_instance_buffer,
            0,
            bytemuck::cast_slice(
                self.model_instances
                    .iter()
                    .map(|instance| instance.to_raw())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
        );
    }

    // Modify the particular instance in the instance buffer
    pub fn update_instance(&mut self, instance_index: usize, instance: instance::Instance) {
        self.model_instances[instance_index] = instance;

        let data = self.model_instances[instance_index].to_raw();
        self.queue.write_buffer(
            &self.model_instance_buffer,
            (instance_index * INSTANCE_RAW_SIZE) as u64,
            bytemuck::cast_slice(&[data]),
        );
    }

    // Modify all the instances of a particular object
    pub fn update_instances(
        &mut self,
        object_index: usize,
        mut instances: Vec<instance::Instance>,
    ) {
        // If the size of the new instances is greater than the range of the current instances
        // For the object, then disregard those instances and create a new set of instances
        // FIXME: find a better way to handle this
        if instances.len() as u32 > self.models[object_index].get_num_instances() {
            self.create_instances(object_index, instances);
            return;
        }

        let offset = self.models[object_index].get_instances().start;

        // If the object is mapped to the default instance of the world origin then
        // create new instances as to not mess with the default instance
        if offset == 0 {
            self.create_instances(object_index, instances);
            return;
        }

        let size = instances.len();

        for i in (0..size).rev() {
            let instance = instances.remove(i);

            self.model_instances[i + offset as usize] = instance;
        }

        let data = {
            let mut d = Vec::with_capacity(size);

            for j in offset..(offset + size as u32) {
                d.push(self.model_instances[j as usize].to_raw());
            }

            d
        };

        self.queue.write_buffer(
            &self.model_instance_buffer,
            offset as u64 * INSTANCE_RAW_SIZE as u64,
            bytemuck::cast_slice(data.as_ref()),
        );
    }

    /// Creates an object and adds it to the scene
    ///
    /// # Arguments
    ///
    /// * `model_path` - Filepath to the model
    /// * `instances` - A vector of instaces with transformation data
    ///
    /// # Returns
    ///
    /// A `usize` index to the objects index in the renderers object directory
    pub fn create_object<P>(&mut self, model_path: P, instances: Vec<instance::Instance>) -> usize
    where
        P: AsRef<Path>,
    {
        let index = self.models.len();
        self.models
            .push(Model::from_obj(model_path, &self.device, &self.queue).unwrap());

        self.update_instances(index, instances);

        index
    }

    /// Function to add a camera to the scene to be rendererd
    pub fn add_camera(
        &mut self,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) {
        self.camera = Camera::new(&self.device, eye, target, up, aspect, fovy, znear, zfar);
        self.queue.write_buffer(
            &self.camera.get_buffer(),
            0,
            bytemuck::cast_slice(&[*self.camera.get_uniform()]),
        );
        self.camera_active = true;
    }

    pub fn update_camera(
        &mut self,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) {
        self.camera.eye = eye;
        self.camera.target = target;
        self.camera.up = up;
        self.camera.aspect = aspect;
        self.camera.fovy = fovy;
        self.camera.znear = znear;
        self.camera.zfar = zfar;

        self.camera.update_view_proj();

        self.queue.write_buffer(
            &self.camera.get_buffer(),
            0,
            bytemuck::cast_slice(&[*self.camera.get_uniform()]),
        );
    }

    /// Function to remove the camera from the scene to stop rendering
    pub fn remove_camera(&mut self) {
        self.camera_active = false;
    }

    pub fn new(window: Arc<Window>) -> Self {
        let instance = Self::create_gpu_instance();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_capabilities = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let config = Self::create_surface_config(size, surface_capabilities);
        surface.configure(&device, &config);

        let camera = Camera::new(
            &device,
            (5.0, 5.0, 5.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        // let camera_controller = CameraController::new(0.2);

        let depth_texture = HeliumTexture::create_depth_texture(&device, &config);

        // The default instance for all models will be at the world origin
        // change the location by creating instances and adding them to this vector
        let model_instances = vec![instance::Instance::default()];

        let model_instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model instance buffer"),
            contents: bytemuck::cast_slice(&[model_instances[0].to_raw()]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        // TODO: Fix this ugly generic
        let render_pipeline = HeliumRenderPipeline::construct_from_layouts::<ModelVertex, &str>(
            vec![
                &HeliumTexture::get_layout(&device),
                &Camera::get_camera_layout(&device),
            ],
            &device,
            &config,
            String::from("Model"),
            "./helium_renderer/src/shaders/vertex_shader.wgsl",
            "./helium_renderer/src/shaders/fragment_shader.wgsl",
        );

        let obj_models = Vec::new();
        // obj_models.push(Model::from_obj("./assets/suzzane.obj", &device, &queue).unwrap());

        Self {
            surface,
            device,
            queue,
            config,
            camera,
            camera_active: false,
            depth_texture,
            render_pipeline,
            models: obj_models,
            model_instances,
            model_instance_buffer,
        }
    }

    // Internal private functions for setting up the GPU
    fn create_gpu_instance() -> Instance {
        Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        })
    }

    fn create_adapter(instance: Instance, surface: &Surface) -> Adapter {
        block_on(instance.request_adapter(&RequestAdapterOptionsBase {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap()
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        smol::block_on(adapter.request_device(
            &DeviceDescriptor {
                required_features: Features::empty(),
                required_limits: Limits::default(),
                label: None,
                ..Default::default()
            },
            None,
        ))
        .unwrap()
    }

    fn create_surface_config(
        size: PhysicalSize<u32>,
        surface_capabilities: SurfaceCapabilities,
    ) -> SurfaceConfiguration {
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|texture_format| texture_format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    // Call this when resizing the window
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
        self.depth_texture = HeliumTexture::create_depth_texture(&self.device, &self.config);

        info!("Resized to: {:?}", new_size);
    }

    // Run any state updates here
    pub fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera);
        // self.camera.update_view_proj();
        // self.queue.write_buffer(
        //     &self.camera.get_buffer(),
        //     0,
        //     bytemuck::cast_slice(&[*self.camera.get_uniform()]),
        // );
    }

    // Call this to handle input
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // self.camera_controller.process_events(event)
        false
    }

    // Call this when requesting redraw
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::WHITE),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.get_view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Only render the scene if the camera is active
            if self.camera_active {
                // Set the render pipeline to the model render pipeline
                render_pass.set_pipeline(self.render_pipeline.as_ref());
                // Set this to the current held instance buffer that stores all the instance data for each mesh
                render_pass.set_vertex_buffer(1, self.model_instance_buffer.slice(..));

                // Sets each of the bind groups
                use model::draw_model::DrawModel;
                for model in self.models.iter() {
                    // Render each mesh in the model with its corresponding material
                    for mesh in model.get_meshes().iter() {
                        render_pass.draw_mesh(
                            mesh,
                            &model.get_materials()[*(mesh.get_material_index().unwrap())],
                            self.camera.get_bind_group(),
                        );
                    }
                }
            }
        }

        self.queue.submit(once(encoder.finish()));
        output.present();

        Ok(())
    }
}
