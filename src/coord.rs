#[derive(Copy, Clone, Debug)]
pub struct Coord(pub i8, pub i8);

impl Coord {
    pub fn dist(&self, other: &Self) -> f32 {
        let dx = (other.0 - self.0) as f32;
        let dy = (other.1 - self.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    // Returns the angle between self and other, closest to prev_angle
    pub fn angle(&self, other: &Self, prev_angle: f32) -> f32 {
        let dx = (other.0 - self.0) as f32;
        let dy = (other.1 - self.1) as f32;
        let atan = dy.atan2(dx);

        let a = atan - prev_angle; // relative to prev_angle

        use std::f32::consts::PI;
        let b = (a + PI).rem_euclid(2. * PI) - PI; // a in [-PI, PI]

        prev_angle + b // prev_angle offset by at most PI
    }
}

#[test]
fn test_angle() {
    use approx::assert_relative_eq;
    use std::f32::consts::{FRAC_PI_2, PI};

    assert_relative_eq!(Coord(0, 0).angle(&Coord(1, 0), 0.), 0.);
    assert_relative_eq!(Coord(1, 0).angle(&Coord(2, 0), 0.), 0.);
    assert_relative_eq!(Coord(1, 0).angle(&Coord(1, 1), 0.), FRAC_PI_2);
    assert_relative_eq!(Coord(0, 0).angle(&Coord(1, 0), PI + 0.1), 2. * PI);
    assert_relative_eq!(Coord(0, 0).angle(&Coord(-1, 0), 0.1), PI);
    assert_relative_eq!(Coord(0, 0).angle(&Coord(-1, 0), -0.1), -PI);
    assert_relative_eq!(Coord(0, 0).angle(&Coord(-1, 0), 2. * PI - 0.1), PI);
    assert_relative_eq!(Coord(0, 0).angle(&Coord(-1, 0), 2. * PI + 0.1), 3. * PI);
    assert_relative_eq!(
        Coord(-1, 0).angle(&Coord(-1, -1), PI / 2. + 0.1),
        3. / 2. * PI
    );
    assert_relative_eq!(Coord(-1, 0).angle(&Coord(-1, -1), PI / 2. - 0.1), -PI / 2.);
}
