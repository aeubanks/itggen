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
    HorizonSingles,
    HorizonDoubles,
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
            "horizon-singles" => Ok(Style::HorizonSingles),
            "horizon-doubles" => Ok(Style::HorizonDoubles),
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
            Style::HorizonSingles => 9,
            Style::HorizonDoubles => 18,
        }
    }

    pub fn sm_string(&self) -> &str {
        match self {
            Style::ItgSingles => "dance-single",
            Style::ItgDoubles => "dance-double",
            Style::ItgTriples => "triple-single",
            Style::PumpSingles => "pump-single",
            Style::PumpDoubles => "pump-double",
            Style::HorizonSingles => "horizon-single",
            Style::HorizonDoubles => "horizon-double",
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

    pub fn init_pos(&self) -> Coord {
        (self.coord(self.init_col(Foot::Left)) + self.coord(self.init_col(Foot::Right))) * 0.5
    }

    pub fn max_x_coord(&self) -> f32 {
        match self {
            Style::ItgSingles | Style::PumpSingles | Style::HorizonSingles => 2.0,
            Style::ItgDoubles | Style::PumpDoubles | Style::HorizonDoubles => 5.0,
            Style::ItgTriples => 8.0,
        }
    }

    pub fn center_x(&self) -> f32 {
        self.max_x_coord() / 2.0
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
        }
    }
}
