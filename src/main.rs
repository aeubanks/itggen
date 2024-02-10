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

    #[structopt(short = "i", help = "Style to base charts off of (e.g. 'itg-singles')")]
    from_style: Style,

    #[structopt(
        short = "o",
        min_values = 1,
        use_delimiter = true,
        help = "Style(s) to generate (e.g. 'pump-doubles,horizon-singles')"
    )]
    to_style: Vec<Style>,

    #[structopt(short, help = "Remove existing autogen charts before generating")]
    remove_existing_autogen: bool,

    #[structopt(short, help = "Preserve arrow jacks/changes from input chart")]
    preserve_input_repetitions: bool,

    #[structopt(
        short,
        parse(from_occurrences),
        help = "Allow crossovers (specify multiple times for harder crossovers)"
    )]
    crossovers: i32,

    #[structopt(
        long = "more-easy-crossovers",
        help = "Generate more but easier crossovers"
    )]
    more_easy_crossovers: bool,

    #[structopt(long = "vroom", help = "Move more on doubles")]
    vroom: bool,

    #[structopt(short, help = "Allow footswitches")]
    footswitches: bool,

    #[structopt(long = "min", help = "Skip difficulties below")]
    min_difficulty: Option<i32>,

    #[structopt(long = "max", help = "Skip difficulties above")]
    max_difficulty: Option<i32>,

    #[structopt(short, help = "Create autogen charts as edits")]
    edits: bool,

    #[structopt(short = "x", help = "Extra string to add to description")]
    extra_description: Option<String>,

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

fn create_params(
    crossovers: i32,
    more_easy_crossovers: bool,
    vroom: bool,
    preserve_input_repetitions: bool,
    disallow_footswitch: bool,
    min_difficulty: Option<i32>,
    max_difficulty: Option<i32>,
) -> GeneratorParameters {
    let has_crossovers = crossovers != 0;
    GeneratorParameters {
        seed: None,
        disallow_footswitch,
        max_repeated: None,
        repeated_decay: Some((1, 0.2)),
        other_foot_repeat_decay: Some(0.3),
        max_dist_between_feet: Some(2.9),
        max_dist_between_feet_if_crossover: Some(2.5),
        dist_between_feet_decay: None,
        max_dist_between_steps: Some(if has_crossovers { 2.9 } else { 2.1 }),
        dist_between_steps_decay: Some((1.5, 0.3)),
        max_horizontal_dist_between_steps: if has_crossovers { None } else { Some(1.0) },
        horizontal_dist_between_steps_decay: None,
        max_horizontal_dist_between_steps_if_crossover: if more_easy_crossovers {
            Some(1.9)
        } else {
            None
        },
        max_vertical_dist_between_steps: None,
        vertical_dist_between_steps_decay: None,
        horizontal_dist_between_3_steps_same_foot_decay: if has_crossovers { None } else { None },
        max_horizontal_dist_between_4_steps_both_feet: if has_crossovers {
            None
        } else {
            Some(2.5)
        },
        horizontal_dist_between_3_steps_decay: Some((
            1.0,
            if has_crossovers || vroom { 0.4 } else { 0.3 },
        )),
        max_angle: Some(PI * (2 + crossovers) as f32 / 4.0),
        angle_decay: None,
        max_turn: Some(if crossovers > 1 { PI } else { PI * 3.0 / 4.0 }),
        turn_decay: None,
        crossover_multiplier: if more_easy_crossovers {
            Some(2.0)
        } else {
            None
        },
        max_bar_angle: None,
        bar_angle_decay: Some((0.0, if has_crossovers { 0.4 } else { 0.2 })),
        preserve_input_repetitions: if preserve_input_repetitions {
            Some(if has_crossovers { 0.001 } else { 0.0 })
        } else {
            None
        },
        doubles_movement: Some((0.5, 0.02)),
        doubles_dist_from_side: if vroom { Some(0.0) } else { None },
        doubles_steps_per_dist: if vroom { Some(4.0) } else { None },
        doubles_track_individual_feet: !vroom && !has_crossovers,
        disallow_foot_opposite_side: !has_crossovers,
        remove_jumps: has_crossovers,
        min_difficulty,
        max_difficulty,
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::from_args();

    let params = create_params(
        opts.crossovers,
        opts.more_easy_crossovers,
        opts.vroom,
        opts.preserve_input_repetitions,
        !opts.footswitches,
        opts.min_difficulty,
        opts.max_difficulty,
    );

    println!("params: {:?}", params);

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
            match sm::generate(
                &contents,
                opts.from_style,
                *to_style,
                params,
                opts.edits,
                opts.extra_description.as_ref(),
            ) {
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
