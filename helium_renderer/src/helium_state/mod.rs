// std
use std::{iter::once, sync::Arc};

// Async
use smol::block_on;

// wgpu imports
use wgpu::{
    Adapter, Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    Instance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RequestAdapterOptionsBase, StoreOp, Surface, SurfaceCapabilities,
    SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor,
};

// winit imports
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

// logging
use log::*;

// State handling modules
mod camera;
mod helium_texture;
mod model;
mod resources;

// module imports
use camera::{Camera, CameraController};
use helium_texture::HeliumTexture;
use model::{model_vertex::ModelVertex, render_pipeline::HeliumRenderPipeline, Model};

pub struct HeliumState {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    // wgpu rendering varables

    // Camera
    camera: Camera,
    camera_controller: CameraController,

    // Depth texture for rendering the correct faces of a mesh
    depth_texture: HeliumTexture,

    // current pipeline for rendering
    render_pipeline: Arc<RenderPipeline>,

    // Models to render
    models: Vec<Model>,
}

impl HeliumState {
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

        let camera_controller = CameraController::new(0.2);

        let depth_texture = HeliumTexture::create_depth_texture(&device, &config);

        // TODO: Fix this ugly generic
        let render_pipeline = HeliumRenderPipeline::construct_from_layouts::<ModelVertex, &str>(
            vec![&HeliumTexture::get_layout(&device), camera.get_layout()],
            &device,
            &config,
            String::from("Model"),
            "./helium_renderer/src/shaders/vertex_shader.wgsl",
            "./helium_renderer/src/shaders/fragment_shader.wgsl",
        );

        let mut obj_models = Vec::new();
        obj_models.push(Model::from_obj("./assets/suzzane.obj", &device, &queue).unwrap());

        Self {
            surface,
            device,
            queue,
            config,
            camera,
            camera_controller,
            depth_texture,
            render_pipeline,
            models: obj_models,
        }
    }

    pub fn create_gpu_instance() -> Instance {
        Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        })
    }

    pub fn create_adapter(instance: Instance, surface: &Surface) -> Adapter {
        block_on(instance.request_adapter(&RequestAdapterOptionsBase {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap()
    }

    pub fn create_device(adapter: &Adapter) -> (Device, Queue) {
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

    pub fn create_surface_config(
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
        self.camera_controller.update_camera(&mut self.camera);
        self.camera.update_view_proj();
        self.queue.write_buffer(
            &self.camera.get_buffer(),
            0,
            bytemuck::cast_slice(&[*self.camera.get_uniform()]),
        );
    }

    // Call this to handle input
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
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

            // Set the render pipeline to the model render pipeline
            render_pass.set_pipeline(self.render_pipeline.as_ref());

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

        self.queue.submit(once(encoder.finish()));
        output.present();

        Ok(())
    }
}
