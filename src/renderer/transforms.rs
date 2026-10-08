#![allow(dead_code)]
use cgmath::*;
use std::f32::consts::PI;

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub fn create_view(
    camera_position: Point3<f32>,
    look_direction: Point3<f32>,
    up_direction: Vector3<f32>,
) -> Matrix4<f32> {
    Matrix4::look_at_rh(camera_position, look_direction, up_direction)
}

pub fn create_projection(aspect: f32) -> Matrix4<f32> {
    let project_mat: Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0 * PI / 5.0), aspect, 0.01, 1000.0);
    project_mat
}

pub fn create_view_projection(
    camera_position: Point3<f32>,
    look_direction: Point3<f32>,
    up_direction: Vector3<f32>,
    aspect: f32,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);
    let project_mat: Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0 * PI / 5.0), aspect, 0.01, 1000.0);
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
}
pub fn create_view_rotation(
    camera_position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    up_direction: Vector3<f32>,
    aspect: f32,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let forward = Vector3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();

    let target = camera_position + forward;

    let view_mat = Matrix4::look_at_rh(camera_position, target, up_direction);
    let project_mat: Matrix4<f32>;
    project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0 * PI / 5.0), aspect, 0.01, 1000.0);
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
}

pub fn create_perspective_projection(
    fovy: Rad<f32>,
    aspect: f32,
    near: f32,
    far: f32,
) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(fovy, aspect, near, far)
}

pub fn create_projection_ortho(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far)
}

pub fn create_view_projection_ortho(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
    camera_position: Point3<f32>,
    look_direction: Point3<f32>,
    up_direction: Vector3<f32>,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);
    let project_mat = OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far);
    let view_project_mat = project_mat * view_mat;
    (view_mat, project_mat, view_project_mat)
}

pub fn create_transforms(
    translation: [f32; 3],
    rotation: [f32; 3],
    scaling: [f32; 3],
) -> Matrix4<f32> {
    let trans_mat =
        Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
    let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
    let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
    let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
    let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);
    let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;
    model_mat
}

pub fn fov_to_projection(fov: openxr::Fovf, near: f32, far: f32) -> Matrix4<f32> {
    let tan_left = fov.angle_left.tan();
    let tan_right = fov.angle_right.tan();
    let tan_down = fov.angle_down.tan();
    let tan_up = fov.angle_up.tan();

    let tan_width = tan_right - tan_left;
    let tan_height = tan_up - tan_down;

    let m00 = 2.0 / tan_width;
    let m11 = 2.0 / tan_height;
    let m20 = (tan_right + tan_left) / tan_width;
    let m21 = (tan_up + tan_down) / tan_height;
    let m22 = -(far + near) / (far - near);
    let m32 = -(2.0 * far * near) / (far - near);

    let opengl_proj = Matrix4::new(
        m00, 0.0, 0.0, 0.0, 0.0, m11, 0.0, 0.0, m20, m21, m22, -1.0, 0.0, 0.0, m32, 0.0,
    );

    OPENGL_TO_WGPU_MATRIX * opengl_proj
}

pub fn pose_to_view_matrix(pose: openxr::Posef) -> Matrix4<f32> {
    let pos = Vector3::new(pose.position.x, pose.position.y, pose.position.z);
    let rot = Quaternion::new(
        pose.orientation.w,
        pose.orientation.x,
        pose.orientation.y,
        pose.orientation.z,
    );

    let translation = Matrix4::from_translation(pos);
    let rotation = Matrix4::from(rot);
    let eye_world = translation * rotation;

    eye_world.invert().unwrap_or(Matrix4::identity())
}

pub fn get_eye_view_matrix(
    player_pos: Vector3<f32>,
    player_yaw: f32,
    xr_eye_pose: openxr::Posef,
) -> Matrix4<f32> {
    let player_translation = Matrix4::from_translation(player_pos);
    let player_rotation = Matrix4::from_angle_y(cgmath::Rad(player_yaw));
    let player_rig_transform = player_translation * player_rotation;

    let eye_pos = Vector3::new(
        xr_eye_pose.position.x,
        xr_eye_pose.position.y,
        xr_eye_pose.position.z,
    );
    let eye_rot = Quaternion::new(
        xr_eye_pose.orientation.w,
        xr_eye_pose.orientation.x,
        xr_eye_pose.orientation.y,
        xr_eye_pose.orientation.z,
    );
    let eye_local_transform = Matrix4::from_translation(eye_pos) * Matrix4::from(eye_rot);
    let eye_world_transform = player_rig_transform * eye_local_transform;

    eye_world_transform.invert().unwrap_or(Matrix4::identity())
}

pub fn quaternion_to_euler(q: Quaternion<f32>) -> (f32, f32, f32) {
    let w = q.s;
    let x = q.v.x;
    let y = q.v.y;
    let z = q.v.z;

    let sinp = 2.0 * (w * x - y * z);
    let pitch = if sinp.abs() >= 1.0 {
        sinp.signum() * std::f32::consts::FRAC_PI_2
    } else {
        sinp.asin()
    };

    let siny_cosp = 2.0 * (w * y + z * x);
    let cosy_cosp = 1.0 - 2.0 * (x * x + y * y);
    let yaw = siny_cosp.atan2(cosy_cosp);

    let sinr_cosp = 2.0 * (w * z + x * y);
    let cosr_cosp = 1.0 - 2.0 * (y * y + z * z);
    let roll = sinr_cosp.atan2(cosr_cosp);

    (pitch, yaw, roll)
}
