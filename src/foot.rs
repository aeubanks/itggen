#[derive(Copy, Clone, Debug)]
pub enum Foot {
    Left,
    Right,
}

impl Foot {
    pub fn other(&self) -> Self {
        match self {
            Foot::Left => Foot::Right,
            Foot::Right => Foot::Left,
        }
    }
}
