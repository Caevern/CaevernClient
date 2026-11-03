use cgmath::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    pub forewards: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub menu: bool,
    pub mouse: Vector2<f32>,
}
impl Default for InputState {
    fn default() -> Self {
        Self {
            forewards: false,
            backwards: false,
            left: false,
            right: false,
            jump: false,
            turn_left: false,
            turn_right: false,
            menu: false,
            mouse: Vector2::new(0.0, 0.0),
        }
    }
}
