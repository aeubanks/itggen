use crate::coord::Coord;
use crate::style::Style;
#[derive(Copy, Clone)]
struct Panel {
    style: Style,
    col: i8,
}

impl Panel {
    pub fn coord(&self) -> Coord {
        match self.style {
            Style::ItgSingles => match self.col {
                0 => Coord(0, 1),
                1 => Coord(1, 0),
                2 => Coord(1, 2),
                3 => Coord(2, 1),
                _ => panic!(),
            },
            Style::ItgDoubles => match self.col {
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
            Style::PumpSingles => match self.col {
                0 => Coord(0, 0),
                1 => Coord(0, 2),
                2 => Coord(1, 1),
                3 => Coord(2, 2),
                4 => Coord(2, 0),
                _ => panic!(),
            },
            Style::PumpDoubles => match self.col {
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
            Style::HorizonSingles => match self.col {
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
            Style::HorizonDoubles => match self.col {
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
