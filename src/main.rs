mod coord;
mod foot;
mod generator;
mod style;

use generator::{Generator, GeneratorParameters};
use style::Style;

fn main() {
    let mut gen = Generator::new(Style::ItgDoubles, GeneratorParameters::default());
    let _ = gen.gen();
    // use coord::Coord;
    // let a = Coord(0, 0);
    // let b = Coord(1, 1);
    // dbg!(a.angle(&b, ));
}
