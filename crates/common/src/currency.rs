#[derive(Clone, Debug)]
pub struct Currency(&'static str);

impl Currency {
    pub fn new(cur: &'static str) -> Self {
        Self(cur)
    }

    pub fn to_string(&self) -> String {
        String::from(self.0)
    }
}

pub const EUR: Currency = Currency("EUR");
