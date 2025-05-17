use crate::coord::Coord;
use crate::foot::Foot;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Style {
    ItgSingles,
    ItgDoubles,
    ItgTriples,
    PumpSingles,
    PumpDoubles,
    PumpTriples,
    PumpHalfDoubles,
    PumpDoublesBrackets,
    PumpMiddleFour,
    HorizonSingles,
    HorizonDoubles,
    HorizonTriples,
    Quads,
    StupidQuads,
}

#[derive(Debug)]
pub struct StyleParseError(String);

impl FromStr for Style {
    type Err = StyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "itg-singles" => Ok(Style::ItgSingles),
            "itg-doubles" => Ok(Style::ItgDoubles),
            "itg-triples" => Ok(Style::ItgTriples),
            "pump-singles" => Ok(Style::PumpSingles),
            "pump-doubles" => Ok(Style::PumpDoubles),
            "pump-triples" => Ok(Style::PumpTriples),
            "pump-halfdoubles" => Ok(Style::PumpHalfDoubles),
            "pump-doubles-brackets" => Ok(Style::PumpDoublesBrackets),
            "pump-middlefour" => Ok(Style::PumpMiddleFour),
            "horizon-singles" => Ok(Style::HorizonSingles),
            "horizon-doubles" => Ok(Style::HorizonDoubles),
            "horizon-triples" => Ok(Style::HorizonTriples),
            "quads" => Ok(Style::Quads),
            "stupid-quads" => Ok(Style::StupidQuads),
            _ => Err(StyleParseError(s.to_owned())),
        }
    }
}

impl ToString for StyleParseError {
    fn to_string(&self) -> String {
        format!("could not parse style '{}'", self.0)
    }
}

impl Style {
    pub fn num_cols(&self) -> i8 {
        match self {
            Style::ItgSingles => 4,
            Style::ItgDoubles => 8,
            Style::ItgTriples => 12,
            Style::PumpSingles => 5,
            Style::PumpDoubles => 10,
            Style::PumpTriples => 15,
            Style::PumpHalfDoubles => 6,
            Style::PumpDoublesBrackets => 10,
            Style::PumpMiddleFour => 4,
            Style::HorizonSingles => 9,
            Style::HorizonDoubles => 18,
            Style::HorizonTriples => 27,
            Style::Quads => 18,
            Style::StupidQuads => 18,
        }
    }

    pub fn extra_0s(&self) -> usize {
        match self {
            Style::PumpHalfDoubles => 2,
            Style::PumpMiddleFour => 3,
            _ => 0,
        }
    }

    pub fn sm_string(&self) -> &str {
        match self {
            Style::ItgSingles => "dance-single",
            Style::ItgDoubles => "dance-double",
            Style::ItgTriples => "dance-triple",
            Style::PumpSingles => "pump-single",
            Style::PumpDoubles
            | Style::PumpHalfDoubles
            | Style::PumpMiddleFour
            | Style::PumpDoublesBrackets => "pump-double",
            Style::PumpTriples => "pump-triple",
            Style::HorizonSingles => "horizon-single",
            Style::HorizonDoubles => "horizon-double",
            Style::HorizonTriples => "horizon-triple",
            Style::Quads => "quads",
            Style::StupidQuads => "stupid-quads",
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
            Style::ItgTriples => match foot {
                Foot::Left => 4,
                Foot::Right => 7,
            },
            Style::PumpSingles => match foot {
                Foot::Left => 0,
                Foot::Right => 4,
            },
            Style::PumpDoubles | Style::StupidQuads => match foot {
                Foot::Left => 4,
                Foot::Right => 5,
            },
            Style::PumpTriples => match foot {
                Foot::Left => 5,
                Foot::Right => 9,
            },
            Style::PumpHalfDoubles => match foot {
                Foot::Left => 2,
                Foot::Right => 3,
            },
            Style::PumpMiddleFour => match foot {
                Foot::Left => 1,
                Foot::Right => 2,
            },
            Style::PumpDoublesBrackets => match foot {
                Foot::Left => 3,
                Foot::Right => 6,
            },
            Style::HorizonSingles => match foot {
                Foot::Left => 1,
                Foot::Right => 7,
            },
            Style::HorizonDoubles => match foot {
                Foot::Left => 7,
                Foot::Right => 10,
            },
            Style::HorizonTriples => match foot {
                Foot::Left => 9,
                Foot::Right => 15,
            },
            Style::Quads => match foot {
                Foot::Left => 9,
                Foot::Right => 10,
            },
        }
    }

    pub fn init_pos(&self) -> Coord {
        (self.coord(self.init_col(Foot::Left)) + self.coord(self.init_col(Foot::Right))) * 0.5
    }

    pub fn max_x_coord(&self) -> f32 {
        match self {
            Style::ItgSingles | Style::PumpSingles | Style::HorizonSingles => 2.0,
            Style::ItgDoubles | Style::PumpDoubles | Style::HorizonDoubles | Style::StupidQuads => {
                5.0
            }
            Style::ItgTriples | Style::PumpTriples | Style::HorizonTriples => 8.0,
            Style::PumpHalfDoubles => 3.0,
            Style::PumpMiddleFour => 1.0,
            Style::PumpDoublesBrackets => 4.0,
            Style::Quads => 11.0,
        }
    }

    pub fn max_y_coord(&self) -> f32 {
        match self {
            Style::StupidQuads => 5.0,
            _ => 2.0,
        }
    }

    pub fn center_x(&self) -> f32 {
        self.max_x_coord() / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.max_y_coord() / 2.0
    }

    pub fn center(&self) -> Coord {
        Coord(self.max_x_coord(), self.max_y_coord()) * 0.5
    }

    pub fn bar_coord(&self) -> Coord {
        Coord(self.center_x(), -0.5)
    }

    pub fn coord(&self, col: i8) -> Coord {
        match self {
            Style::ItgSingles => match col {
                0 => Coord(0.0, 1.0),
                1 => Coord(1.0, 0.0),
                2 => Coord(1.0, 2.0),
                3 => Coord(2.0, 1.0),
                _ => panic!(),
            },
            Style::ItgDoubles => match col {
                0 => Coord(0.0, 1.0),
                1 => Coord(1.0, 0.0),
                2 => Coord(1.0, 2.0),
                3 => Coord(2.0, 1.0),
                4 => Coord(3.0, 1.0),
                5 => Coord(4.0, 0.0),
                6 => Coord(4.0, 2.0),
                7 => Coord(5.0, 1.0),
                _ => panic!(),
            },
            Style::ItgTriples => match col {
                0 => Coord(0.0, 1.0),
                1 => Coord(1.0, 0.0),
                2 => Coord(1.0, 2.0),
                3 => Coord(2.0, 1.0),
                4 => Coord(3.0, 1.0),
                5 => Coord(4.0, 0.0),
                6 => Coord(4.0, 2.0),
                7 => Coord(5.0, 1.0),
                8 => Coord(6.0, 1.0),
                9 => Coord(7.0, 0.0),
                10 => Coord(7.0, 2.0),
                11 => Coord(8.0, 1.0),
                _ => panic!(),
            },
            Style::PumpSingles => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 2.0),
                2 => Coord(1.0, 1.0),
                3 => Coord(2.0, 2.0),
                4 => Coord(2.0, 0.0),
                _ => panic!(),
            },
            Style::PumpDoubles => match col {
                0 => Coord(0.0, 0.2),
                1 => Coord(0.0, 1.8),
                2 => Coord(1.0, 1.0),
                3 => Coord(2.0, 1.8),
                4 => Coord(2.0, 0.2),
                5 => Coord(3.0, 0.2),
                6 => Coord(3.0, 1.8),
                7 => Coord(4.0, 1.0),
                8 => Coord(5.0, 1.8),
                9 => Coord(5.0, 0.2),
                _ => panic!(),
            },
            Style::PumpTriples => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 2.0),
                2 => Coord(1.0, 1.0),
                3 => Coord(2.0, 2.0),
                4 => Coord(2.0, 0.0),
                5 => Coord(3.0, 0.0),
                6 => Coord(3.0, 2.0),
                7 => Coord(4.0, 1.0),
                8 => Coord(5.0, 2.0),
                9 => Coord(5.0, 0.0),
                10 => Coord(6.0, 0.0),
                11 => Coord(6.0, 2.0),
                12 => Coord(7.0, 1.0),
                13 => Coord(8.0, 2.0),
                14 => Coord(8.0, 0.0),
                _ => panic!(),
            },
            Style::PumpHalfDoubles => match col {
                0 => Coord(0.0, 1.0),
                1 => Coord(1.0, 2.0),
                2 => Coord(1.0, 0.0),
                3 => Coord(2.0, 0.0),
                4 => Coord(2.0, 2.0),
                5 => Coord(3.0, 1.0),
                _ => panic!(),
            },
            Style::PumpMiddleFour => match col {
                0 => Coord(0.0, 1.0),
                1 => Coord(0.0, 0.0),
                2 => Coord(1.0, 0.0),
                3 => Coord(1.0, 1.0),
                _ => panic!(),
            },
            Style::PumpDoublesBrackets => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 1.0),
                2 => Coord(1.0, 1.0),
                3 => Coord(1.0, 0.0),
                4 => Coord(2.0, 0.0),
                5 => Coord(2.0, 1.0),
                6 => Coord(3.0, 0.0),
                7 => Coord(3.0, 1.0),
                8 => Coord(4.0, 1.0),
                9 => Coord(4.0, 0.0),
                _ => panic!(),
            },
            Style::HorizonSingles => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 1.0),
                2 => Coord(0.0, 2.0),
                3 => Coord(1.0, 0.0),
                4 => Coord(1.0, 1.0),
                5 => Coord(1.0, 2.0),
                6 => Coord(2.0, 2.0),
                7 => Coord(2.0, 1.0),
                8 => Coord(2.0, 0.0),
                _ => panic!(),
            },
            Style::HorizonDoubles => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 1.0),
                2 => Coord(0.0, 2.0),
                3 => Coord(1.0, 0.0),
                4 => Coord(1.0, 1.0),
                5 => Coord(1.0, 2.0),
                6 => Coord(2.0, 2.0),
                7 => Coord(2.0, 1.0),
                8 => Coord(2.0, 0.0),
                9 => Coord(3.0, 0.0),
                10 => Coord(3.0, 1.0),
                11 => Coord(3.0, 2.0),
                12 => Coord(4.0, 0.0),
                13 => Coord(4.0, 1.0),
                14 => Coord(4.0, 2.0),
                15 => Coord(5.0, 2.0),
                16 => Coord(5.0, 1.0),
                17 => Coord(5.0, 0.0),
                _ => panic!(),
            },
            Style::HorizonTriples => match col {
                0 => Coord(0.0, 0.0),
                1 => Coord(0.0, 1.0),
                2 => Coord(0.0, 2.0),
                3 => Coord(1.0, 0.0),
                4 => Coord(1.0, 1.0),
                5 => Coord(1.0, 2.0),
                6 => Coord(2.0, 2.0),
                7 => Coord(2.0, 1.0),
                8 => Coord(2.0, 0.0),
                9 => Coord(3.0, 0.0),
                10 => Coord(3.0, 1.0),
                11 => Coord(3.0, 2.0),
                12 => Coord(4.0, 0.0),
                13 => Coord(4.0, 1.0),
                14 => Coord(4.0, 2.0),
                15 => Coord(5.0, 2.0),
                16 => Coord(5.0, 1.0),
                17 => Coord(5.0, 0.0),
                18 => Coord(6.0, 0.0),
                19 => Coord(6.0, 1.0),
                20 => Coord(6.0, 2.0),
                21 => Coord(7.0, 0.0),
                22 => Coord(7.0, 1.0),
                23 => Coord(7.0, 2.0),
                24 => Coord(8.0, 2.0),
                25 => Coord(8.0, 1.0),
                26 => Coord(8.0, 0.0),
                _ => panic!(),
            },
            Style::Quads => match col {
                0 => Coord(0.0, 0.2),
                1 => Coord(0.0, 1.8),
                2 => Coord(1.0, 1.0),
                3 => Coord(2.0, 1.8),
                4 => Coord(2.0, 0.2),
                5 => Coord(3.0, 0.2),
                6 => Coord(3.0, 1.8),
                7 => Coord(4.0, 1.0),
                8 => Coord(5.0, 1.8),
                9 => Coord(5.0, 0.2),

                10 => Coord(6.0, 1.0),
                11 => Coord(7.0, 0.0),
                12 => Coord(7.0, 2.0),
                13 => Coord(8.0, 1.0),
                14 => Coord(9.0, 1.0),
                15 => Coord(10.0, 0.0),
                16 => Coord(10.0, 2.0),
                17 => Coord(11.0, 1.0),
                _ => panic!(),
            },
            Style::StupidQuads => match col {
                0 => Coord(0.0, 3.2),
                1 => Coord(0.0, 4.8),
                2 => Coord(1.0, 4.0),
                3 => Coord(2.0, 4.8),
                4 => Coord(2.0, 3.2),
                5 => Coord(3.0, 3.2),
                6 => Coord(3.0, 4.8),
                7 => Coord(4.0, 4.0),
                8 => Coord(5.0, 4.8),
                9 => Coord(5.0, 3.2),

                10 => Coord(0.0, 1.0),
                11 => Coord(1.0, 0.0),
                12 => Coord(1.0, 2.0),
                13 => Coord(2.0, 1.0),
                14 => Coord(3.0, 1.0),
                15 => Coord(4.0, 0.0),
                16 => Coord(4.0, 2.0),
                17 => Coord(5.0, 1.0),
                _ => panic!(),
            },
        }
    }

    pub fn sm_cols_for_col(&self, col: i8) -> Vec<i8> {
        match self {
            Style::PumpDoublesBrackets => match col {
                0 => vec![0, 2],
                1 => vec![1, 2],
                2 => vec![3, 2],
                3 => vec![4, 2],
                4 => vec![4, 5],
                5 => vec![3, 6],
                6 => vec![5, 7],
                7 => vec![6, 7],
                8 => vec![8, 7],
                9 => vec![9, 7],
                _ => panic!(),
            },
            _ => {
                vec![col]
            }
        }
    }
}
