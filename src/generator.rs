use crate::coord::Coord;
use crate::foot::Foot;
use crate::style::Style;
use rand::prelude::*;

#[derive(Copy, Clone, Default)]
pub struct GeneratorParameters {
    pub seed: Option<u64>,
    pub disallow_footswitch: bool,
    pub max_repeated: Option<i32>,
    pub repeated_decay: Option<(i32, f32)>,
    pub max_dist_between_feet: Option<f32>,
    pub dist_between_feet_decay: Option<(f32, f32)>,
    pub max_dist_between_steps: Option<f32>,
    pub dist_between_steps_decay: Option<(f32, f32)>,
    pub max_horizontal_dist_between_3_steps: Option<f32>,
    pub horizontal_dist_between_3_steps_decay: Option<(f32, f32)>,
    pub max_angle: Option<f32>,
    pub angle_decay: Option<(f32, f32)>,
    pub max_turn: Option<f32>,
    pub turn_decay: Option<(f32, f32)>,
    pub preserve_input_repetitions: Option<f32>,
    pub doubles_movement: Option<(f32, f32)>,
    pub disallow_foot_opposite_side: bool,
}

#[derive(Debug, Default, Copy, Clone)]
struct FootStatus {
    pub last_last_col: Option<i8>,
    pub last_col: Option<i8>,
    pub repeated: i32,
    pub last_input_col: Option<i8>,
}

#[derive(Debug, Copy, Clone)]
struct Zone {
    start: Coord,
    end: Coord,
    steps: i32,
    total_steps_until_end: i32,
}

impl Zone {
    fn step(&mut self) {
        self.steps += 1;
    }

    fn is_done(&self) -> bool {
        self.steps >= self.total_steps_until_end
    }

    fn center(&self) -> Coord {
        let ratio = self.steps as f32 / self.total_steps_until_end as f32;
        self.start + (self.end - self.start) * ratio
    }
}

pub struct Generator {
    style: Style,
    params: GeneratorParameters,
    rand: StdRng,
    feet_status: [FootStatus; 2],
    next_foot: Foot,
    prev_angle: f32,
    zone: Zone,
}

impl Generator {
    pub fn new(style: Style, params: GeneratorParameters) -> Self {
        let mut rand = params
            .seed
            .map(|s| StdRng::seed_from_u64(s))
            .unwrap_or_else(|| StdRng::from_entropy());
        let next_foot = if rand.gen() { Foot::Left } else { Foot::Right };
        let zone_end = Self::rand_zone_end(&mut rand, style);
        let zone_steps = Self::rand_zone_total_steps(&mut rand, style.init_pos().dist(zone_end));
        Self {
            style,
            params,
            rand,
            feet_status: [FootStatus::default(); 2],
            next_foot,
            prev_angle: 0.0,
            zone: Zone {
                start: style.init_pos(),
                end: zone_end,
                steps: 0,
                total_steps_until_end: zone_steps,
            },
        }
    }
}

impl Generator {
    #[cfg(test)]
    pub fn gen(&mut self) -> i8 {
        self.gen_with_input_col(-1)
    }

    pub fn gen_with_input_col(&mut self, input_col: i8) -> i8 {
        if self.params.preserve_input_repetitions.is_some() {
            if self.next_foot_status().last_input_col == Some(input_col) {
                if let Some(lc) = self.next_foot_status().last_col {
                    self.step_with_input_col(lc, input_col);
                    return lc;
                }
            } else if self.prev_foot_status().last_input_col == Some(input_col) {
                if let Some(lc) = self.prev_foot_status().last_col {
                    self.step_without_switching_feet(lc, input_col);
                    return lc;
                }
            }
        }
        self.gen_impl(input_col)
    }

    fn gen_impl(&mut self, input_col: i8) -> i8 {
        let col;
        if self.next_foot_status().last_col.is_none() {
            col = self.style.init_col(self.next_foot);
        } else {
            col = self.choose(input_col);
        }
        self.step_impl(col, input_col, true);
        col
    }

    fn choose(&mut self, input_col: i8) -> i8 {
        let col_probs: Vec<(i8, f32)> = self
            .valid_cols()
            .into_iter()
            .map(|c| (c, self.prob_with_input_col(c, input_col)))
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
        let prob_remaining = self.rand.gen_range(0.0, total_prob);
        Self::choose_from_probs_with_prob(col_probs, prob_remaining)
    }

    fn choose_from_probs_with_prob(col_probs: Vec<(i8, f32)>, mut prob: f32) -> i8 {
        for (c, p) in &col_probs {
            prob -= p;
            if prob <= 0.0 {
                return *c;
            }
        }
        col_probs.last().unwrap().0
    }

    #[cfg(test)]
    fn step(&mut self, col: i8) {
        self.step_impl(col, -1, true)
    }

    fn step_with_input_col(&mut self, col: i8, input_col: i8) {
        self.step_impl(col, input_col, true)
    }

    fn step_without_switching_feet(&mut self, col: i8, input_col: i8) {
        self.step_impl(col, input_col, false)
    }

    fn step_impl(&mut self, col: i8, input_col: i8, switch_feet: bool) {
        let foot_status = if switch_feet {
            self.next_foot_status_mut()
        } else {
            self.prev_foot_status_mut()
        };

        if foot_status.last_col == Some(col) {
            foot_status.repeated += 1;
        } else {
            foot_status.repeated = 1;
        }

        foot_status.last_last_col = foot_status.last_col;
        foot_status.last_col = Some(col);
        foot_status.last_input_col = Some(input_col);

        if let Some(a) = self.calc_cur_angle() {
            self.prev_angle = a;
        }

        self.zone.step();

        if self.params.doubles_movement.is_some() {
            if self.zone.is_done() {
                let next_end = self.next_zone_end();
                let next_total_steps = self.next_zone_total_steps(self.zone.end.dist(next_end));
                self.zone = Zone {
                    start: self.zone.end,
                    end: next_end,
                    steps: 0,
                    total_steps_until_end: next_total_steps,
                };
            }
        }

        if switch_feet {
            self.next_foot = self.next_foot.other();
        }
    }

    fn next_zone_end(&mut self) -> Coord {
        Self::rand_zone_end(&mut self.rand, self.style)
    }

    fn next_zone_total_steps(&mut self, dist_from_prev: f32) -> i32 {
        Self::rand_zone_total_steps(&mut self.rand, dist_from_prev)
    }

    fn rand_zone_end(rand: &mut StdRng, style: Style) -> Coord {
        let max = style.max_x_coord() - 0.5;
        if max <= 0.5 {
            return Coord(1.0, 1.0);
        }
        let rx = rand.gen_range(0.5, max);
        Coord(rx, 1.0)
    }

    fn rand_zone_total_steps(rand: &mut StdRng, dist_from_prev: f32) -> i32 {
        let ret = if dist_from_prev <= 1.0 {
            rand.gen_range(4, 8)
        } else {
            rand.gen_range(8, 32)
        };
        ret
    }
}

impl Generator {
    const EPSILON: f32 = 0.00001;

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

    fn test_angle(&self, col: i8) -> Option<f32> {
        let (lc, rc) = match self.next_foot {
            Foot::Left => (col, self.feet_status[Foot::Right as usize].last_col?),
            Foot::Right => (self.feet_status[Foot::Left as usize].last_col?, col),
        };
        let l = self.style.coord(lc);
        let r = self.style.coord(rc);
        Some(l.angle(r, self.prev_angle))
    }

    fn calc_cur_angle(&self) -> Option<f32> {
        let lc = self.feet_status[Foot::Left as usize].last_col?;
        let rc = self.feet_status[Foot::Right as usize].last_col?;
        let l = self.style.coord(lc);
        let r = self.style.coord(rc);
        Some(l.angle(r, self.prev_angle))
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
                if prev_coord.dist(cur_coord) > md + Self::EPSILON {
                    return false;
                }
            }
        }
        if let Some(md) = self.params.max_dist_between_steps {
            if let Some(prev_col) = self.next_foot_status().last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                if prev_coord.dist(cur_coord) > md + Self::EPSILON {
                    return false;
                }
            }
        }
        if let Some(md) = self.params.max_horizontal_dist_between_3_steps {
            if let Some(prev_col) = self.next_foot_status().last_last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                if (prev_coord.0 - cur_coord.0).abs() > md + Self::EPSILON {
                    return false;
                }
            }
        }
        if let Some(ma) = self.params.max_angle {
            if let Some(a) = self.test_angle(col) {
                if a.abs() > ma + Self::EPSILON {
                    return false;
                }
            }
        }
        if let Some(mt) = self.params.max_turn {
            if let Some(a) = self.test_angle(col) {
                if (a - self.prev_angle).abs() > mt + Self::EPSILON {
                    return false;
                }
            }
        }
        if self.params.disallow_foot_opposite_side {
            let coord = self.style.coord(col);
            match self.next_foot {
                Foot::Left => {
                    if coord.0 >= self.style.max_x_coord() - Self::EPSILON {
                        return false;
                    }
                }
                Foot::Right => {
                    if coord.0 <= Self::EPSILON {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[cfg(test)]
    fn prob(&self, col: i8) -> f32 {
        self.prob_with_input_col(col, -1)
    }

    fn prob_with_input_col(&self, col: i8, input_col: i8) -> f32 {
        let mut prob = 1.0;
        if let Some((repeated, decay)) = self.params.repeated_decay {
            if self.next_foot_status().last_col == Some(col) {
                let over_repeated = self.next_foot_status().repeated - repeated;
                if over_repeated > 0 {
                    prob *= decay.powi(over_repeated);
                }
            }
        }
        if let Some((dist, decay)) = self.params.dist_between_feet_decay {
            if let Some(prev_col) = self.prev_foot_status().last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                let over_dist = prev_coord.dist(cur_coord) - dist;
                if over_dist > 0.0 {
                    prob *= decay.powf(over_dist);
                }
            }
        }
        if let Some((dist, decay)) = self.params.dist_between_steps_decay {
            if let Some(prev_col) = self.next_foot_status().last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                let over_dist = prev_coord.dist(cur_coord) - dist;
                if over_dist > 0.0 {
                    prob *= decay.powf(over_dist);
                }
            }
        }
        if let Some((dist, decay)) = self.params.horizontal_dist_between_3_steps_decay {
            if let Some(prev_col) = self.next_foot_status().last_last_col {
                let prev_coord = self.style.coord(prev_col);
                let cur_coord = self.style.coord(col);
                let over_dist = (prev_coord.0 - cur_coord.0).abs() - dist;
                if over_dist > 0.0 {
                    prob *= decay.powf(over_dist);
                }
            }
        }
        if let Some((angle, decay)) = self.params.angle_decay {
            if let Some(a) = self.test_angle(col) {
                let over_angle = a.abs() - angle;
                if over_angle > 0.0 {
                    prob *= decay.powf(over_angle);
                }
            }
        }
        if let Some((turn, decay)) = self.params.turn_decay {
            if let Some(a) = self.test_angle(col) {
                let over_angle = (a - self.prev_angle).abs() - turn;
                if over_angle > 0.0 {
                    prob *= decay.powf(over_angle);
                }
            }
        }
        // if input column is same as previous input column, penalize if same column as before
        if let Some(different_decay) = self.params.preserve_input_repetitions {
            if let Some(last_input_col) = self.next_foot_status().last_input_col {
                if input_col != last_input_col && Some(col) == self.next_foot_status().last_col {
                    prob *= different_decay;
                }
            }
        }
        if let Some((dist, decay)) = self.params.doubles_movement {
            let center = self.zone.center();
            let cur_coord = self.style.coord(col);
            let over_dist = center.dist(cur_coord) - dist;
            if over_dist > 0.0 {
                prob *= decay.powf(over_dist);
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
    {
        let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
        gen.next_foot = Foot::Left;
        assert_eq!(gen.gen(), 0);
        assert_eq!(gen.gen(), 3);
    }
    {
        let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
        gen.next_foot = Foot::Right;
        assert_eq!(gen.gen(), 3);
        assert_eq!(gen.gen(), 0);
    }
}

#[test]
fn state() {
    let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
    let f = gen.next_foot;
    assert_eq!(gen.next_foot_status().last_col, None);
    assert_eq!(gen.prev_foot_status().last_col, None);
    let c = gen.gen();
    assert_ne!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_col, None);
    assert_eq!(gen.prev_foot_status().last_col, Some(c));
}

#[test]
fn preserve_input_repetitions() {
    let mut params = GeneratorParameters::default();
    params.preserve_input_repetitions = Some(1.0);
    let mut gen = Generator::new(Style::HorizonDoubles, params);

    let f = gen.next_foot;
    let c1 = gen.gen_with_input_col(7);
    assert_ne!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_input_col, None);
    assert_eq!(gen.prev_foot_status().last_input_col, Some(7));

    assert_eq!(gen.gen_with_input_col(7), c1);
    assert_ne!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_input_col, None);
    assert_eq!(gen.prev_foot_status().last_input_col, Some(7));

    let c2 = gen.gen_with_input_col(6);
    assert_eq!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_input_col, Some(7));
    assert_eq!(gen.prev_foot_status().last_input_col, Some(6));

    assert_eq!(gen.gen_with_input_col(7), c1);
    assert_ne!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_input_col, Some(6));
    assert_eq!(gen.prev_foot_status().last_input_col, Some(7));

    assert_eq!(gen.gen_with_input_col(6), c2);
    assert_eq!(f, gen.next_foot);
    assert_eq!(gen.next_foot_status().last_input_col, Some(7));
    assert_eq!(gen.prev_foot_status().last_input_col, Some(6));
}

#[test]
fn valid_steps() {
    use std::f32::consts::PI;
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
            let gen = Generator::new(*style, params);
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
        let mut gen = Generator::new(Style::ItgSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(0);
        gen.step(3);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2]);
    }
    // max repeated
    {
        let mut params = GeneratorParameters::default();
        params.max_repeated = Some(2);
        let mut gen = Generator::new(Style::ItgSingles, params);
        gen.next_foot = Foot::Left;
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
        params.max_dist_between_feet = Some(2.0);
        let mut gen = Generator::new(Style::ItgDoubles, params);
        gen.next_foot = Foot::Right;
        gen.step(3);
        gen.step(0);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3]);
        gen.step(7);
        assert_eq!(gen.valid_cols(), vec![4, 5, 6, 7]);
    }
    // max dist same foot steps
    {
        let mut params = GeneratorParameters::default();
        params.max_dist_between_steps = Some(2.0);
        let mut gen = Generator::new(Style::ItgDoubles, params);
        gen.next_foot = Foot::Left;
        gen.step(0);
        gen.step(7);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3]);
        gen.step(0);
        assert_eq!(gen.valid_cols(), vec![4, 5, 6, 7]);
    }
    // max horizontal dist same foot 3 steps
    {
        let mut params = GeneratorParameters::default();
        params.max_horizontal_dist_between_3_steps = Some(1.5);
        let mut gen = Generator::new(Style::HorizonSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(2);
        gen.step(8);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
        gen.step(5);
        gen.step(8);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3, 4, 5]);
        gen.step(6);
        gen.step(8);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
        gen.step(5);
        gen.step(8);
        assert_eq!(gen.valid_cols(), vec![3, 4, 5, 6, 7, 8]);
    }
    // max angle
    {
        let mut params = GeneratorParameters::default();
        params.max_angle = Some(PI * 3.0 / 4.0);
        let mut gen = Generator::new(Style::HorizonSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(1);
        gen.step(1);
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3, 5]);
        gen.next_foot = Foot::Right;
        gen.step(7);
        gen.step(7);
        assert_eq!(gen.valid_cols(), vec![3, 5, 6, 7, 8]);
    }
    // max turn
    {
        let mut params = GeneratorParameters::default();
        params.max_turn = Some(PI / 2.0);
        let mut gen = Generator::new(Style::HorizonSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(3);
        gen.step(4);
        assert_eq!(gen.valid_cols(), vec![0, 1, 3, 4, 7, 8]);

        gen.step(5);
        gen.step(4);
        assert_eq!(gen.valid_cols(), vec![1, 2, 4, 5, 6, 7]);
    }
    // foot other side
    {
        let mut params = GeneratorParameters::default();
        params.disallow_foot_opposite_side = true;
        let mut gen = Generator::new(Style::ItgDoubles, params);
        gen.next_foot = Foot::Left;
        assert_eq!(gen.valid_cols(), vec![0, 1, 2, 3, 4, 5, 6]);
        gen.next_foot = Foot::Right;
        assert_eq!(gen.valid_cols(), vec![1, 2, 3, 4, 5, 6, 7]);
    }
}

#[test]
fn steps_prob() {
    use approx::assert_relative_eq;
    use std::f32::consts::PI;
    // repeated decay
    {
        let mut params = GeneratorParameters::default();
        params.repeated_decay = Some((2, 0.5));
        let mut gen = Generator::new(Style::ItgSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(0);
        gen.step(3);
        gen.step(0);
        gen.step(3);
        assert_eq!(gen.prob(0), 1.0);
        gen.step(0);
        assert_eq!(gen.prob(3), 1.0);
        gen.step(3);
        assert_eq!(gen.prob(0), 0.5);
        gen.step(0);
        assert_eq!(gen.prob(3), 0.5);
        gen.step(3);
        assert_eq!(gen.prob(0), 0.25);
        gen.step(0);
        assert_eq!(gen.prob(3), 0.25);
    }
    // dist between feet decay
    {
        let mut params = GeneratorParameters::default();
        params.dist_between_feet_decay = Some((1.0, 0.5));
        let mut gen = Generator::new(Style::ItgDoubles, params);
        gen.next_foot = Foot::Right;
        gen.step(4);
        gen.step(3);
        assert_eq!(gen.prob(0), 0.5);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(7), 0.25);
    }
    // dist between foot steps decay
    {
        let mut params = GeneratorParameters::default();
        params.dist_between_steps_decay = Some((1.0, 0.5));
        let mut gen = Generator::new(Style::ItgDoubles, params);
        gen.next_foot = Foot::Left;
        gen.step(3);
        gen.step(5);
        assert_eq!(gen.prob(0), 0.5);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(7), 0.25);
    }
    // horizontal dist between 3 foot steps decay
    {
        let mut params = GeneratorParameters::default();
        params.horizontal_dist_between_3_steps_decay = Some((1.0, 0.5));
        let mut gen = Generator::new(Style::HorizonSingles, params);
        gen.next_foot = Foot::Left;
        gen.step(2);
        gen.step(8);
        assert_eq!(gen.prob(0), 1.0);
        assert_eq!(gen.prob(1), 1.0);
        assert_eq!(gen.prob(2), 1.0);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(5), 1.0);
        assert_eq!(gen.prob(6), 1.0);
        assert_eq!(gen.prob(7), 1.0);
        assert_eq!(gen.prob(8), 1.0);
        gen.step(5);
        gen.step(8);
        assert_eq!(gen.prob(0), 1.0);
        assert_eq!(gen.prob(1), 1.0);
        assert_eq!(gen.prob(2), 1.0);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(5), 1.0);
        assert_eq!(gen.prob(6), 0.5);
        assert_eq!(gen.prob(7), 0.5);
        assert_eq!(gen.prob(8), 0.5);
        gen.step(6);
        gen.step(8);
        assert_eq!(gen.prob(0), 1.0);
        assert_eq!(gen.prob(1), 1.0);
        assert_eq!(gen.prob(2), 1.0);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(5), 1.0);
        assert_eq!(gen.prob(6), 1.0);
        assert_eq!(gen.prob(7), 1.0);
        assert_eq!(gen.prob(8), 1.0);
        gen.step(5);
        gen.step(8);
        assert_eq!(gen.prob(0), 0.5);
        assert_eq!(gen.prob(1), 0.5);
        assert_eq!(gen.prob(2), 0.5);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(5), 1.0);
        assert_eq!(gen.prob(6), 1.0);
        assert_eq!(gen.prob(7), 1.0);
        assert_eq!(gen.prob(8), 1.0);
    }
    // angle decay
    {
        {
            let mut params = GeneratorParameters::default();
            params.angle_decay = Some((PI / 2.0, 0.5));
            let mut gen = Generator::new(Style::HorizonSingles, params);
            gen.next_foot = Foot::Left;
            gen.step(1);
            gen.step(1);
        }
        {
            let mut params = GeneratorParameters::default();
            params.angle_decay = Some((PI / 2.0, 0.5));
            let mut gen = Generator::new(Style::HorizonSingles, params);
            gen.next_foot = Foot::Right;
            gen.step(7);
            gen.step(7);
            assert_relative_eq!(gen.prob(6), 1.0);
            assert_relative_eq!(gen.prob(7), 1.0);
            assert_relative_eq!(gen.prob(8), 1.0);
            assert_relative_eq!(gen.prob(3), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(5), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(1), 0.5_f32.powf(PI / 2.0));
        }
    }
    // turn decay
    {
        {
            let mut params = GeneratorParameters::default();
            params.turn_decay = Some((PI / 2.0, 0.5));
            let mut gen = Generator::new(Style::HorizonSingles, params);
            gen.next_foot = Foot::Left;
            gen.step(1);
            gen.step(1);
            assert_relative_eq!(gen.prob(0), 1.0);
            assert_relative_eq!(gen.prob(1), 1.0);
            assert_relative_eq!(gen.prob(2), 1.0);
            assert_relative_eq!(gen.prob(3), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(5), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(7), 0.5_f32.powf(PI / 2.0));
        }
        {
            let mut params = GeneratorParameters::default();
            params.turn_decay = Some((PI / 2.0, 0.5));
            let mut gen = Generator::new(Style::HorizonSingles, params);
            gen.next_foot = Foot::Right;
            gen.step(7);
            gen.step(7);
            assert_relative_eq!(gen.prob(6), 1.0);
            assert_relative_eq!(gen.prob(7), 1.0);
            assert_relative_eq!(gen.prob(8), 1.0);
            assert_relative_eq!(gen.prob(3), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(5), 0.5_f32.powf(PI / 4.0));
            assert_relative_eq!(gen.prob(1), 0.5_f32.powf(PI / 2.0));
        }
    }
    // preserve input repetitions different decay
    {
        let mut params = GeneratorParameters::default();
        params.preserve_input_repetitions = Some(0.5);
        let mut gen = Generator::new(Style::ItgSingles, params);
        gen.next_foot = Foot::Left;
        gen.step_with_input_col(0, 4);
        gen.step_with_input_col(3, 8);
        assert_eq!(gen.prob_with_input_col(0, 4), 1.0);
        assert_eq!(gen.prob_with_input_col(0, 5), 0.5);
        assert_eq!(gen.prob_with_input_col(1, 4), 1.0);
        assert_eq!(gen.prob_with_input_col(1, 5), 1.0);
    }
    // doubles movement distance decay
    {
        let mut params = GeneratorParameters::default();
        params.doubles_movement = Some((1.0, 0.5));
        let mut gen = Generator::new(Style::HorizonDoubles, params);
        gen.next_foot = Foot::Left;
        gen.step(1);
        gen.step(7);
        gen.zone = Zone {
            start: Coord(0.0, 1.0),
            end: Coord(5.0, 1.0),
            steps: 0,
            total_steps_until_end: 5,
        };
        assert_eq!(gen.prob(0), 1.0);
        assert_eq!(gen.prob(1), 1.0);
        assert_eq!(gen.prob(2), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(7), 0.5);
        assert_eq!(gen.prob(10), 0.25);
        gen.step(1);
        assert_eq!(gen.prob(1), 1.0);
        assert_eq!(gen.prob(3), 1.0);
        assert_eq!(gen.prob(4), 1.0);
        assert_eq!(gen.prob(5), 1.0);
        assert_eq!(gen.prob(7), 1.0);
        assert_eq!(gen.prob(10), 0.5);
    }
}

#[test]
fn test_prev_angle() {
    use approx::assert_relative_eq;
    use std::f32::consts::PI;
    let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
    gen.next_foot = Foot::Left;
    gen.step(0);
    gen.step(3);
    assert_relative_eq!(gen.prev_angle, 0.0);
    gen.step(2);
    assert_relative_eq!(gen.prev_angle, -PI / 4.0);
    gen.step(1);
    assert_relative_eq!(gen.prev_angle, -PI / 2.0);
    gen.step(0);
    assert_relative_eq!(gen.prev_angle, -PI / 4.0);
    gen.step(2);
    assert_relative_eq!(gen.prev_angle, PI / 4.0);
    gen.step(3);
    assert_relative_eq!(gen.prev_angle, 3.0 / 4.0 * PI);
    gen.step(0);
    assert_relative_eq!(gen.prev_angle, PI);
}
