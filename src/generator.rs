use crate::foot::Foot;
use crate::style::Style;
use rand::prelude::*;

#[derive(Copy, Clone)]
pub struct GeneratorParameters {
    seed: Option<u64>,
    allow_footswitch: bool,
}

impl Default for GeneratorParameters {
    fn default() -> Self {
        Self {
            seed: None,
            allow_footswitch: false,
        }
    }
}

#[derive(Default, Copy, Clone)]
struct FootStatus {
    pub last_col: Option<i8>,
    pub repeated: i32,
}

pub struct Generator {
    style: Style,
    params: GeneratorParameters,
    rand: StdRng,
    feet_status: [FootStatus; 2],
    next_foot: Foot,
}

impl Generator {
    pub fn new(style: Style, params: GeneratorParameters) -> Self {
        let mut rand = params
            .seed
            .map(|s| StdRng::seed_from_u64(s))
            .unwrap_or_else(|| StdRng::from_entropy());
        let next_foot = if rand.gen() { Foot::Left } else { Foot::Right };
        Self {
            style,
            params,
            rand,
            feet_status: [FootStatus::default(); 2],
            next_foot,
        }
    }
}

impl Generator {
    #[must_use]
    pub fn gen(&mut self) -> i8 {
        let col;
        if self.feet_status[self.next_foot as usize].last_col.is_none() {
            col = self.style.init_col(self.next_foot);
        } else {
            col = self.choose();
        }
        self.step(col);
        col
    }

    fn choose(&mut self) -> i8 {
        let col_probs: Vec<(i8, f32)> = self
            .valid_cols()
            .into_iter()
            .map(|c| (c, self.prob(c)))
            .collect();
        self.random(col_probs)
    }

    fn valid_cols(&self) -> Vec<i8> {
        let cols: Vec<i8> = (0..(self.style.num_cols()))
            .filter(|c| self.is_valid_col(*c))
            .collect();
        if cols.is_empty() {
            panic!("no available columns!");
        }
        cols
    }

    fn random(&mut self, col_probs: Vec<(i8, f32)>) -> i8 {
        let total_prob: f32 = col_probs.iter().map(|(_, p)| p).sum();
        let mut prob_remaining = self.rand.gen_range(0., total_prob);
        for (c, p) in &col_probs {
            prob_remaining -= p;
            if prob_remaining <= 0. {
                return *c;
            }
        }
        col_probs.last().unwrap().0
    }

    fn step(&mut self, col: i8) {
        if self.next_foot_status().last_col == Some(col) {
            self.next_foot_status_mut().repeated += 1;
        } else {
            self.next_foot_status_mut().repeated = 0;
        }
        self.next_foot_status_mut().last_col = Some(col);

        self.next_foot = self.next_foot.other();
    }
}

impl Generator {
    fn next_foot_status(&self) -> &FootStatus {
        &self.feet_status[self.next_foot as usize]
    }

    fn prev_foot_status(&self) -> &FootStatus {
        &self.feet_status[self.next_foot.other() as usize]
    }

    fn next_foot_status_mut(&mut self) -> &mut FootStatus {
        &mut self.feet_status[self.next_foot as usize]
    }

    fn prev_foot_status_mut(&mut self) -> &mut FootStatus {
        &mut self.feet_status[self.next_foot.other() as usize]
    }

    fn is_valid_col(&self, col: i8) -> bool {
        if !self.params.allow_footswitch {
            if self.prev_foot_status().last_col == Some(col) {
                return false;
            }
        }
        true
    }

    fn prob(&self, col: i8) -> f32 {
        1.
    }
}

#[test]
fn sanity() {
    let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
    for _ in 0..100 {
        let _ = gen.gen();
    }
}

#[test]
fn first_steps() {
    let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
    let c1 = gen.gen();
    let c2 = gen.gen();
    assert!((c1 == 0 && c2 == 3) || (c1 == 3 && c2 == 0));
}

#[test]
fn valid_steps() {
    {
        let gen = Generator {
            style: Style::ItgSingles,
            params: GeneratorParameters {
                seed: None,
                allow_footswitch: true,
            },
            rand: StdRng::from_entropy(),
            feet_status: [
                FootStatus {
                    last_col: Some(0),
                    repeated: 0,
                },
                FootStatus {
                    last_col: Some(3),
                    repeated: 0,
                },
            ],
            next_foot: Foot::Left,
        };
        let valid_cols = gen.valid_cols();
        assert_eq!(valid_cols, vec![0, 1, 2, 3]);
    }
    // no footswitches
    {
        let gen = Generator {
            style: Style::ItgSingles,
            params: GeneratorParameters {
                seed: None,
                allow_footswitch: false,
            },
            rand: StdRng::from_entropy(),
            feet_status: [
                FootStatus {
                    last_col: Some(0),
                    repeated: 0,
                },
                FootStatus {
                    last_col: Some(3),
                    repeated: 0,
                },
            ],
            next_foot: Foot::Left,
        };
        let valid_cols = gen.valid_cols();
        assert_eq!(valid_cols, vec![0, 1, 2]);
    }
}
