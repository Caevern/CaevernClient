use crate::renderer::{init_wgpu, render::Assets};
use image::{DynamicImage, GenericImageView};

pub struct TextureObject {
    pub texture: wgpu::Texture,
    pub texture_size: wgpu::Extent3d,
    pub texture_rgba: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,
}
impl TextureObject {
    pub fn create(path: &str, init: &init_wgpu::InitWgpu) -> Self {
        let texture_data = Assets::get(path).expect("Failed to load embedded texture");
        let img = image::load_from_memory(&texture_data.data).expect("Failed to load texture");
        println!("loaded {}", path);
        let texture_rgba = img.to_rgba8().to_vec();
        let (width, height) = img.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture: wgpu::Texture = init.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            texture,
            texture_size,
            texture_rgba,
            texture_width: width,
            texture_height: height,
        }
    }

    pub fn load_from_dynamic_image(img: DynamicImage, init: &init_wgpu::InitWgpu) -> Self {
        let texture_rgba = img.to_rgba8().to_vec();
        let (width, height) = img.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture: wgpu::Texture = init.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            texture,
            texture_size,
            texture_rgba,
            texture_width: width,
            texture_height: height,
        }
    }
}
