use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Coord(pub f32, pub f32);

impl Coord {
    pub fn dist(&self, other: &Self) -> f32 {
        let dx = other.0 - self.0;
        let dy = other.1 - self.1;
        (dx * dx + dy * dy).sqrt()
    }

    // Returns the angle between self and other, closest to prev_angle
    pub fn angle(&self, other: &Self, prev_angle: f32) -> f32 {
        let dx = other.0 - self.0;
        let dy = other.1 - self.1;
        let atan = dy.atan2(dx);

        use std::f32::consts::PI;
        let rotations = (prev_angle - atan + PI).div_euclid(2. * PI); // rotations to adjust atan result by
        atan + rotations * 2. * PI
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f32> for Coord {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

#[test]
fn test_angle() {
    use approx::assert_relative_eq;
    use std::f32::consts::{FRAC_PI_2, PI};

    assert_relative_eq!(Coord(0.0, 0.0).angle(&Coord(1.0, 0.0), 0.), 0.);
    assert_relative_eq!(Coord(1.0, 0.0).angle(&Coord(2.0, 0.0), 0.), 0.);
    assert_relative_eq!(Coord(1.0, 0.0).angle(&Coord(1.0, 1.0), 0.), FRAC_PI_2);
    assert_relative_eq!(Coord(0.0, 0.0).angle(&Coord(1.0, 0.0), PI + 0.1), 2. * PI);
    assert_relative_eq!(Coord(0.0, 0.0).angle(&Coord(-1.0, 0.0), 0.1), PI);
    assert_relative_eq!(Coord(0.0, 0.0).angle(&Coord(-1.0, 0.0), -0.1), -PI);
    assert_relative_eq!(Coord(0.0, 0.0).angle(&Coord(-1.0, 0.0), 2. * PI - 0.1), PI);
    assert_relative_eq!(
        Coord(0.0, 0.0).angle(&Coord(-1.0, 0.0), 2. * PI + 0.1),
        3. * PI
    );
    assert_relative_eq!(
        Coord(-1.0, 0.0).angle(&Coord(-1.0, -1.0), PI / 2. + 0.1),
        3. / 2. * PI
    );
    assert_relative_eq!(
        Coord(-1.0, 0.0).angle(&Coord(-1.0, -1.0), PI / 2. - 0.1),
        -PI / 2.
    );

    let a = Coord(0.0, 1.0);
    let b = Coord(1.0, 2.0);
    assert_eq!(a.angle(&b, 0.221), a.angle(&b, 0.2));
}
