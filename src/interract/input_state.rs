#[derive(Default, Debug, Clone, Copy)]
pub struct InputState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub left: bool,
    pub right: bool,
    pub menu: bool,
}
