use crate::foot::Foot;
use crate::style::Style;
use rand::prelude::*;

#[derive(Copy, Clone, Default)]
pub struct GeneratorParameters {
    seed: Option<u64>,
    disallow_footswitch: bool,
    max_repeated: Option<i32>,
    repeated_decay: Option<(i32, f32)>,
    max_dist_between_feet: Option<f32>,
}

#[derive(Debug, Default, Copy, Clone)]
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
        self.choose_from_probs(col_probs)
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

    fn choose_from_probs(&mut self, col_probs: Vec<(i8, f32)>) -> i8 {
        let total_prob: f32 = col_probs.iter().map(|(_, p)| p).sum();
        let prob_remaining = self.rand.gen_range(0., total_prob);
        Self::choose_from_probs_with_prob(col_probs, prob_remaining)
    }

    fn choose_from_probs_with_prob(col_probs: Vec<(i8, f32)>, mut prob: f32) -> i8 {
        for (c, p) in &col_probs {
            prob -= p;
            if prob <= 0. {
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

    fn is_valid_col(&self, col: i8) -> bool {
        if self.params.disallow_footswitch {
            if self.prev_foot_status().last_col == Some(col) {
                return false;
            }
        }
        if let Some(mr) = self.params.max_repeated {
            if self.next_foot_status().last_col == Some(col)
                && self.next_foot_status().repeated >= mr
            {
                return false;
            }
        }
        if let Some(md) = self.params.max_dist_between_feet {
            if let Some(prev_col) = self.prev_foot_status().last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                if prev_coord.dist(&cur_coord) + std::f32::EPSILON > md {
                    return false;
                }
            }
        }
        true
    }

    fn prob(&self, col: i8) -> f32 {
        let mut prob = 1.;
        if let Some((r, d)) = self.params.repeated_decay {
            if self.next_foot_status().last_col == Some(col) {
                let over_repeated = self.next_foot_status().repeated - r;
                if over_repeated > 0 {
                    prob *= d.powi(over_repeated);
                }
            }
        }
        prob
    }
}

#[test]
fn test_choose_from_probs_with_prob() {
    assert_eq!(
        Generator::choose_from_probs_with_prob(vec![(5, 0.1)], 0.05),
        5
    );
    assert_eq!(
        Generator::choose_from_probs_with_prob(vec![(5, 0.1), (6, 0.1)], 0.05),
        5
    );
    assert_eq!(
        Generator::choose_from_probs_with_prob(vec![(5, 0.1), (6, 0.1)], 0.15),
        6
    );
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
        for style in &[
            Style::ItgSingles,
            Style::ItgDoubles,
            Style::PumpSingles,
            Style::PumpDoubles,
            Style::HorizonSingles,
            Style::HorizonDoubles,
        ] {
            let params = GeneratorParameters::default();
            let gen = Generator {
                style: *style,
                params,
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
            assert_eq!(
                gen.valid_cols(),
                (0..(style.num_cols())).collect::<Vec<i8>>()
            );
        }
    }
    // no footswitches
    {
        let mut params = GeneratorParameters::default();
        params.disallow_footswitch = true;
        let gen = Generator {
            style: Style::ItgSingles,
            params,
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
        assert_eq!(gen.valid_cols(), vec![0, 1, 2]);
    }
    // max repeated
    {
        let mut params = GeneratorParameters::default();
        params.max_repeated = Some(2);
        let mut gen = Generator {
            style: Style::ItgSingles,
            params,
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
        gen.step(0);
        gen.step(3);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3]);
        gen.step(0);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3]);
        gen.step(3);
        assert_eq!(gen.valid_cols(), vec![1, 2, 3]);
        gen.step(0);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2]);
    }
    // max dist two feet
    {
        let mut params = GeneratorParameters::default();
        params.max_dist_between_feet = Some(2.);
        let mut gen = Generator {
            style: Style::ItgDoubles,
            params,
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
            next_foot: Foot::Right,
        };
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3]);
        gen.step(7);
        assert_eq!(gen.valid_cols(), vec![4, 5, 6, 7]);
    }
}

#[test]
fn steps_prob() {
    {
        let mut params = GeneratorParameters::default();
        params.repeated_decay = Some((2, 0.5));
        let mut gen = Generator {
            style: Style::ItgSingles,
            params,
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
        gen.step(0);
        gen.step(3);
        gen.step(0);
        gen.step(3);
        assert_eq!(gen.prob(0), 1.);
        gen.step(0);
        assert_eq!(gen.prob(3), 1.);
        gen.step(3);
        assert_eq!(gen.prob(0), 0.5);
        gen.step(0);
        assert_eq!(gen.prob(3), 0.5);
        gen.step(3);
        assert_eq!(gen.prob(0), 0.25);
        gen.step(0);
        assert_eq!(gen.prob(3), 0.25);
    }
}
