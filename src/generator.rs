use crate::foot::Foot;
use crate::style::Style;
use rand::prelude::*;

#[derive(Copy, Clone, Default)]
pub struct GeneratorParameters {
    seed: Option<u64>,
}

#[derive(Default, Copy, Clone)]
struct FootStatus {
    pub last_col: Option<i8>,
    pub repeated: i32,
}

pub struct Generator {
    style: Style,
    rand: rand::rngs::StdRng,
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
            rand,
            feet_status: [FootStatus::default(); 2],
            next_foot,
        }
    }
}

impl Generator {
    pub fn gen(&mut self) -> i8 {
        let col;
        if self.feet_status[self.next_foot as usize].last_col.is_none() {
            col = self.style.init_col(self.next_foot);
        } else {
            col = 0
        }
        self.step(col);
        col
    }

    fn step(&mut self, col: i8) {
        if self.feet_status[self.next_foot as usize].last_col == Some(col) {
            self.feet_status[self.next_foot as usize].repeated += 1;
        } else {
            self.feet_status[self.next_foot as usize].repeated = 0;
        }
        self.feet_status[self.next_foot as usize].last_col = Some(col);

        self.next_foot = self.next_foot.other();
    }
}

#[test]
fn first_steps() {
    let mut gen = Generator::new(Style::ItgSingles, GeneratorParameters::default());
    let c1 = gen.gen();
    let c2 = gen.gen();
    assert!((c1 == 0 && c2 == 3) || (c1 == 3 && c2 == 0));
}
