mod coord;
mod foot;
mod generator;
mod sm;
mod style;

use generator::{Generator, GeneratorParameters};
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use style::Style;

#[derive(Debug, StructOpt)]
#[structopt(name = "itggen")]
struct Opts {
    #[structopt(
        parse(from_os_str),
        min_values = 1,
        help = "Paths of/directories containing .sm files to generate charts"
    )]
    inputs: Vec<PathBuf>,

    #[structopt(
        short,
        long = "from",
        help = "Style to base charts off of (e.g. 'itg-singles')"
    )]
    from_style: Style,

    #[structopt(
        short,
        long = "to",
        min_values = 1,
        use_delimiter = true,
        help = "Style(s) to generate (e.g. 'pump-doubles,horizon-singles')"
    )]
    to_style: Vec<Style>,

    #[structopt(short, help = "Remove existing autogen charts before generating")]
    remove_existing_autogen: bool,

    #[structopt(short, help = "Preserve arrow jacks/changes from input chart")]
    preserve_input_repetitions: bool,

    #[structopt(short, help = "Use crossover parameters")]
    crossover_params: bool,

    #[structopt(short, help = "Dry run (don't actually write to disk)")]
    dry_run: bool,
}

fn sm_files(path: &Path) -> Vec<PathBuf> {
    let rd = match std::fs::read_dir(&path) {
        Ok(rd) => rd,
        Err(_) => {
            return vec![];
        }
    };
    let mut ret = Vec::new();
    for de in rd {
        if let Ok(de) = de {
            if let Ok(t) = de.file_type() {
                if t.is_dir() {
                    ret.append(&mut sm_files(&de.path()));
                } else if t.is_file() {
                    let p = de.path();
                    if let Some(Some(ext)) = p.extension().map(|e| e.to_str()) {
                        if ext.to_lowercase() == "sm" {
                            ret.push(de.path());
                        }
                    }
                }
            }
        }
    }
    ret
}

fn normal_params(preserve_input_repetitions: bool) -> GeneratorParameters {
    GeneratorParameters {
        seed: None,
        disallow_footswitch: true,
        max_repeated: None,
        repeated_decay: Some((1, 0.2)),
        max_dist_between_feet: Some(2.9),
        dist_between_feet_decay: None,
        max_dist_between_steps: None,
        dist_between_steps_decay: Some((1.5, 0.3)),
        max_horizontal_dist_between_steps: Some(1.0),
        horizontal_dist_between_steps_decay: None,
        max_vertical_dist_between_steps: None,
        vertical_dist_between_steps_decay: None,
        max_horizontal_dist_between_3_steps: None,
        horizontal_dist_between_3_steps_decay: Some((1.0, 0.3)),
        max_angle: Some(PI / 2.0),
        angle_decay: None,
        max_turn: None,
        turn_decay: None,
        max_bar_angle: None,
        bar_angle_decay: Some((0.0, 0.1)),
        preserve_input_repetitions: if preserve_input_repetitions {
            Some(0.0)
        } else {
            None
        },
        doubles_movement: Some((1.2, 0.1)),
        disallow_foot_opposite_side: true,
    }
}

fn crossover_params(preserve_input_repetitions: bool) -> GeneratorParameters {
    GeneratorParameters {
        seed: None,
        disallow_footswitch: true,
        max_repeated: None,
        repeated_decay: Some((1, 0.2)),
        max_dist_between_feet: Some(2.9),
        dist_between_feet_decay: None,
        max_dist_between_steps: None,
        dist_between_steps_decay: Some((1.5, 0.3)),
        max_horizontal_dist_between_steps: Some(1.0),
        horizontal_dist_between_steps_decay: None,
        max_vertical_dist_between_steps: None,
        vertical_dist_between_steps_decay: None,
        max_horizontal_dist_between_3_steps: None,
        horizontal_dist_between_3_steps_decay: Some((1.0, 0.3)),
        max_angle: Some(PI * 3.0 / 4.0),
        angle_decay: None,
        max_turn: None,
        turn_decay: None,
        max_bar_angle: None,
        bar_angle_decay: None,
        preserve_input_repetitions: if preserve_input_repetitions {
            Some(0.001)
        } else {
            None
        },
        doubles_movement: Some((1.2, 0.1)),
        disallow_foot_opposite_side: false,
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::from_args();

    let params = if opts.crossover_params {
        crossover_params(opts.preserve_input_repetitions)
    } else {
        normal_params(opts.preserve_input_repetitions)
    };

    let files: Vec<PathBuf> = opts.inputs.iter().flat_map(|i| sm_files(&i)).collect();

    if files.is_empty() {
        println!("no input files...");
    }

    for p in files {
        println!("generating for {:?}", p);
        let mut contents = match std::fs::read_to_string(p.clone()) {
            Ok(s) => s,
            Err(e) => {
                println!("  couldn't read file: {}", e);
                continue;
            }
        };
        if opts.remove_existing_autogen {
            match sm::remove_existing_autogen(&contents) {
                Ok(s) => {
                    contents = s;
                }
                Err(e) => {
                    println!("  couldn't remove existing autogen: {}", e);
                    continue;
                }
            }
        }
        let mut generated = String::new();
        for to_style in &opts.to_style {
            println!("  {:?} -> {:?}", opts.from_style, to_style);
            match sm::generate(&contents, opts.from_style, *to_style, params) {
                Ok(s) => {
                    generated.push('\n');
                    generated.push_str(&s);
                }
                Err(e) => {
                    println!("  skipped: {}", e);
                }
            }
        }
        contents.push_str(&generated);
        if opts.dry_run {
            println!("  done (dry run)");
        } else {
            std::fs::write(p.clone(), contents)?;
            println!("  done");
        }
    }

    let mut gen = Generator::new(Style::ItgDoubles, GeneratorParameters::default());
    let _ = gen.gen_with_input_col(0);

    Ok(())
}
