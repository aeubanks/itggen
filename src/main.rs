mod coord;
mod foot;
mod generator;
mod style;

use generator::{Generator, GeneratorParameters};
use std::path::PathBuf;
use structopt::StructOpt;
use style::Style;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,
}

fn main() {
    let opts = Opts::from_args();

    for p in opts.inputs {
        let _ = std::fs::read_to_string(p).unwrap();
    }

    let mut gen = Generator::new(Style::ItgDoubles, GeneratorParameters::default());
    let _ = gen.gen();
    let _ = gen.gen_with_input_col(0);
}
