mod coord;
mod foot;
mod generator;
mod sm;
mod style;

use generator::{Generator, GeneratorParameters};
use std::path::PathBuf;
use structopt::StructOpt;
use style::Style;

#[derive(Debug, StructOpt)]
#[structopt(name = "itggen")]
struct Opts {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,

    #[structopt(short)]
    from_style: Style,

    #[structopt(short)]
    to_style: Style,
}

fn main() -> std::io::Result<()> {
    use std::f32::consts::PI;

    let opts = Opts::from_args();

    let params = GeneratorParameters {
        seed: None,
        disallow_footswitch: true,
        max_repeated: None,
        repeated_decay: Some((2, 0.2)),
        max_dist_between_feet: Some(2.9),
        dist_between_feet_decay: None,
        max_dist_between_steps: Some(2.0),
        dist_between_steps_decay: Some((1.5, 0.3)),
        max_horizontal_dist_between_3_steps: None,
        horizontal_dist_between_3_steps_decay: Some((1.5, 0.2)),
        max_angle: Some(PI / 2.0),
        angle_decay: None,
        max_turn: None,
        turn_decay: None,
        max_bar_angle: None,
        bar_angle_decay: Some((0.0, 0.3)),
        preserve_input_repetitions: Some(0.1),
        doubles_movement: Some((1.5, 0.2)),
        disallow_foot_opposite_side: true,
    };

    for p in opts.inputs {
        println!("generating for {:?}", p);
        let mut contents = std::fs::read_to_string(p.clone())?;
        match sm::generate(&contents, opts.from_style, opts.to_style, params) {
            Ok(s) => {
                contents.push('\n');
                contents.push_str(&s);
                std::fs::write(p, contents)?;
                println!("  done");
            }
            Err(e) => {
                println!("  skipped: {}", e);
            }
        }
    }

    let mut gen = Generator::new(Style::ItgDoubles, GeneratorParameters::default());
    let _ = gen.gen_with_input_col(0);

    Ok(())
}
