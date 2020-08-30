#[derive(Copy, Clone)]
pub struct Coord(pub i8, pub i8);

impl Coord {
    pub fn dist(&self, other: &Self) -> f32 {
        let dx = (self.0 - other.0) as f32;
        let dy = (self.1 - other.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}
