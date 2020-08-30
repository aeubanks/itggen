use crate::coord::Coord;
use crate::foot::Foot;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Style {
    ItgSingles,
    ItgDoubles,
    PumpSingles,
    PumpDoubles,
    HorizonSingles,
    HorizonDoubles,
}

impl Style {
    pub fn num_cols(&self) -> i8 {
        match self {
            Style::ItgSingles => 4,
            Style::ItgDoubles => 8,
            Style::PumpSingles => 5,
            Style::PumpDoubles => 10,
            Style::HorizonSingles => 9,
            Style::HorizonDoubles => 18,
        }
    }

    pub fn init_col(&self, foot: Foot) -> i8 {
        match self {
            Style::ItgSingles => match foot {
                Foot::Left => 0,
                Foot::Right => 3,
            },
            Style::ItgDoubles => match foot {
                Foot::Left => 3,
                Foot::Right => 4,
            },
            Style::PumpSingles => match foot {
                Foot::Left => 0,
                Foot::Right => 4,
            },
            Style::PumpDoubles => match foot {
                Foot::Left => 4,
                Foot::Right => 5,
            },
            Style::HorizonSingles => match foot {
                Foot::Left => 1,
                Foot::Right => 7,
            },
            Style::HorizonDoubles => match foot {
                Foot::Left => 7,
                Foot::Right => 10,
            },
        }
    }
    pub fn coord(&self, col: i8) -> Coord {
        match self {
            Style::ItgSingles => match col {
                0 => Coord(0, 1),
                1 => Coord(1, 0),
                2 => Coord(1, 2),
                3 => Coord(2, 1),
                _ => panic!(),
            },
            Style::ItgDoubles => match col {
                0 => Coord(0, 1),
                1 => Coord(1, 0),
                2 => Coord(1, 2),
                3 => Coord(2, 1),
                4 => Coord(3, 1),
                5 => Coord(4, 0),
                6 => Coord(4, 2),
                7 => Coord(5, 1),
                _ => panic!(),
            },
            Style::PumpSingles => match col {
                0 => Coord(0, 0),
                1 => Coord(0, 2),
                2 => Coord(1, 1),
                3 => Coord(2, 2),
                4 => Coord(2, 0),
                _ => panic!(),
            },
            Style::PumpDoubles => match col {
                0 => Coord(0, 0),
                1 => Coord(0, 2),
                2 => Coord(1, 1),
                3 => Coord(2, 2),
                4 => Coord(2, 0),
                5 => Coord(3, 0),
                6 => Coord(3, 2),
                7 => Coord(4, 1),
                8 => Coord(5, 2),
                9 => Coord(5, 0),
                _ => panic!(),
            },
            Style::HorizonSingles => match col {
                0 => Coord(0, 0),
                1 => Coord(0, 1),
                2 => Coord(0, 2),
                3 => Coord(1, 0),
                4 => Coord(1, 1),
                5 => Coord(1, 2),
                6 => Coord(2, 2),
                7 => Coord(2, 1),
                8 => Coord(2, 0),
                _ => panic!(),
            },
            Style::HorizonDoubles => match col {
                0 => Coord(0, 0),
                1 => Coord(0, 1),
                2 => Coord(0, 2),
                3 => Coord(1, 0),
                4 => Coord(1, 1),
                5 => Coord(1, 2),
                6 => Coord(2, 2),
                7 => Coord(2, 1),
                8 => Coord(2, 0),
                9 => Coord(3, 0),
                10 => Coord(3, 1),
                11 => Coord(3, 2),
                12 => Coord(4, 0),
                13 => Coord(4, 1),
                14 => Coord(4, 2),
                15 => Coord(5, 2),
                16 => Coord(5, 1),
                17 => Coord(5, 0),
                _ => panic!(),
            },
        }
    }
}
