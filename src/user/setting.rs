pub struct CameraSetting {
    pub walk_speed: f32,
    pub mouse_sensitivity: f32,
    pub render_distance: u8,
}

impl Default for CameraSetting {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            mouse_sensitivity: 0.5,
            render_distance: 16,
        }
    }
}