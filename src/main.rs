mod coord;
mod foot;
mod generator;
mod style;

use generator::Generator;
use style::Style;

fn main() {
    let mut gen = Generator::new(Style::ItgDoubles);
    let _ = gen.gen();
}
