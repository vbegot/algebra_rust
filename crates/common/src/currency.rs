#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Currency(&'static str);

impl Currency {
    pub fn new(cur: &'static str) -> Self {
        Self(cur)
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub const EUR: Currency = Currency("EUR");
