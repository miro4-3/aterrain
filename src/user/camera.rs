use std::f32::consts::PI;

use macroquad::input::{ KeyCode, get_keys_down, mouse_delta_position, set_cursor_grab, show_mouse };
use macroquad::math::{ Vec3, vec3 };
use macroquad::window::{ screen_width, screen_height };
use macroquad::camera::{ Camera3D, Projection };

use crate::user::setting::CameraSetting;

pub struct Camera {
    pub position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 2.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn update(&mut self, setting: &CameraSetting, dt: f32) {
        self.look_with_mouse(setting.mouse_sensitivity);
        self.keyboard(setting, dt);
    }
    fn look_with_mouse(&mut self, sensitivity: f32) {
        let delta = mouse_delta_position();

        self.yaw -= delta.x * sensitivity;
        self.pitch += delta.y * sensitivity;

        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
    }

    fn keyboard(&mut self, setting: &CameraSetting, dt: f32) {
        let keys = get_keys_down();
        let forward = self.forward();
        let right = self.right();
        let speed = setting.walk_speed * dt;

        if keys.contains(&KeyCode::W) {
            self.position += forward * speed;
        }
        if keys.contains(&KeyCode::S) {
            self.position -= forward * speed;
        }
        if keys.contains(&KeyCode::A) {
            self.position -= right * speed;
        }
        if keys.contains(&KeyCode::D) {
            self.position += right * speed;
        }
        if keys.contains(&KeyCode::LeftShift) {
            self.position.y -= speed;
        }
        if keys.contains(&KeyCode::Space) {
            self.position.y += speed;
        }
        if keys.contains(&KeyCode::Escape) {
            set_cursor_grab(false);
            show_mouse(true);
        }
        if keys.contains(&KeyCode::Key1) {
            set_cursor_grab(true);
            show_mouse(false);
        }
    }

    pub fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos()
        ).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalize()
    }

    pub fn camera_3d(&self) -> Camera3D {
        let target = self.position + self.forward();
        Camera3D {
            position: self.position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            fovy: (90.0_f32).to_radians(),
            aspect: Some(screen_width() / screen_height()),
            z_near: 0.1,
            z_far: f32::MAX,
            projection: Projection::Perspective,
            render_target: None,
            viewport: None,
        }
    }
}
