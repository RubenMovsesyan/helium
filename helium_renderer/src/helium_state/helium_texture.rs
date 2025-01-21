// std imports
use std::sync::Arc;

// wgpu imports
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, CompareFunction,
    Device, Extent3d, FilterMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, SurfaceConfiguration, TexelCopyBufferLayout, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension,
};

// image imports
use image::{load_from_memory, GenericImageView, ImageError};

// logging
use log::*;

// Constants
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

// In the bind group, binding 0 is the texture, and binding 1 is the sampler
// only visible in the fragment shader
const HELIUM_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Texture bind group layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };

pub struct HeliumTextureBindGroupLayout(Arc<BindGroupLayout>);

impl HeliumTextureBindGroupLayout {
    // Creates a bind group layout from the default descriptor
    // Use this in the texture to make sure all the layouts are correct
    pub fn from_device(device: &Device) -> Self {
        Self(Arc::new(device.create_bind_group_layout(
            &HELIUM_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR,
        )))
    }

    pub fn get_layout(&self) -> Arc<BindGroupLayout> {
        self.0.clone()
    }
}

impl Clone for HeliumTextureBindGroupLayout {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct HeliumTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
    layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>,
}

impl HeliumTexture {
    pub fn get_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&HELIUM_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    pub fn get_bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }

    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8]) -> Result<Self, ImageError> {
        let img = load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());
        // TODO: Add support for changing the texture filter modes
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let layout = device.create_bind_group_layout(&HELIUM_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        debug!("texture size: {:?}", size);
        // Write the texture to the queue
        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Ok(Self {
            texture,
            view,
            sampler,
            layout: Some(layout),
            bind_group: Some(bind_group),
        })
    }

    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> Self {
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let description = TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&description);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            layout: None,
            bind_group: None,
        }
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }
}
