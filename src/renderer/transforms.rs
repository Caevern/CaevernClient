#![allow(dead_code)]
use std::{f32::consts::PI, sync::Arc};
use winit::window::Window;
use cgmath::*;

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
pub struct InitWgpu<'window> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> InitWgpu<'window> {
    pub async fn init_wgpu(window: &Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter:false,
            })
            .await
            .expect("No GPU adapter found that can present to this surface!");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode:surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
     
        Self {
            instance,
            surface,
            device,
            queue,
            config,
            size,
        }
    }
}

pub fn create_view(camera_position: Point3<f32>, look_direction: Point3<f32>, up_direction: Vector3<f32>) -> Matrix4<f32> {
    Matrix4::look_at_rh(camera_position, look_direction, up_direction)
}

pub fn create_projection(aspect:f32) -> Matrix4<f32> {
    let project_mat:Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.01, 1000.0);
    project_mat
}

pub fn create_view_projection(camera_position: Point3<f32>, look_direction: Point3<f32>, up_direction: Vector3<f32>,
    aspect:f32) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);     
    let project_mat:Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.01, 1000.0);
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
}
pub fn create_view_rotation(camera_position: Point3<f32>, yaw: f32, pitch: f32, up_direction: Vector3<f32>,
    aspect:f32) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    
    let forward = Vector3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    ).normalize();

    let target = camera_position + forward;

    let view_mat = Matrix4::look_at_rh(camera_position, target, up_direction);     
    let project_mat:Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.01, 1000.0);
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
}

pub fn create_perspective_projection(fovy:Rad<f32>, aspect:f32, near: f32, far:f32) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(fovy, aspect, near, far)
}

pub fn create_projection_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far)    
}

pub fn create_view_projection_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32, camera_position: Point3<f32>, 
    look_direction: Point3<f32>, up_direction: Vector3<f32>) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);     
    let project_mat = OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far);    
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
} 

pub fn create_transforms(translation:[f32; 3], rotation:[f32; 3], scaling:[f32; 3]) -> Matrix4<f32> {
    let trans_mat = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
    let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
    let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
    let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
    let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);
    let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;
    model_mat
}