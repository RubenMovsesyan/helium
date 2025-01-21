use helium_io::load_shader;

use std::{borrow::Cow, path::Path, sync::Arc};

use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Face, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, StencilState, SurfaceConfiguration, VertexState,
};

use super::{instance::InstanceRaw, vertex::Vertex};
use crate::helium_state::helium_texture::DEPTH_FORMAT;
pub struct HeliumRenderPipeline(Arc<RenderPipeline>);

impl HeliumRenderPipeline {
    pub fn construct_from_layouts<V, P>(
        layouts: Vec<&BindGroupLayout>,
        device: &Device,
        config: &SurfaceConfiguration,
        name: String,
        vertex_shader_path: P,
        fragment_shader_path: P,
    ) -> Arc<RenderPipeline>
    where
        V: Vertex,
        P: AsRef<Path>,
    {
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&(name.clone() + " Render Pipeline Layout")),
            bind_group_layouts: layouts.as_slice(),
            push_constant_ranges: &[],
        });

        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&(name.clone() + " Vertex Shader")),
            source: ShaderSource::Wgsl(Cow::from(
                load_shader(vertex_shader_path).unwrap().as_str(),
            )),
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&(name.clone() + " Fragment Shader")),
            source: ShaderSource::Wgsl(Cow::from(
                load_shader(fragment_shader_path).unwrap().as_str(),
            )),
        });

        Arc::new(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&(name + " Render Pipeline")),
            layout: Some(&layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: Some("main"),
                buffers: &[V::desc(), InstanceRaw::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: Some("main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // Change this to make a wireframe
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        }))
    }
}
