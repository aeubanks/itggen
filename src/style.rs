#[derive(Copy, Clone)]
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
}
