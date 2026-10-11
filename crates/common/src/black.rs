use statrs::distribution::{ContinuousCDF, Normal};

pub enum CallPut {
    Call,
    Put,
}

pub fn black_price(
    forward: f64,
    discount: f64,
    volatility: f64,
    maturity: f64,
    strike: f64,
    call_put: CallPut,
) -> f64 {
    let w = match call_put {
        CallPut::Call => 1.0,
        CallPut::Put => -1.0,
    };
    let stdev = volatility * maturity.sqrt();
    let d1 = (forward / strike).ln() / stdev + 0.5 * stdev;
    let d2 = d1 - stdev;
    let normal = Normal::new(0.0, 1.0).unwrap();
    discount * (w * forward * normal.cdf(w * d1) - w * strike * normal.cdf(w * d2))
}
