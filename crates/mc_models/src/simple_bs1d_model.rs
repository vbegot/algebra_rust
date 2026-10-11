// Simple BS 1d model with constant vol, constant rate

use common::currency::Currency;

pub struct ModelParameters {
    pub underlying: String,
    pub spot: f64,
    pub volatility: f64,
    pub rate: f64,
    pub currency: Currency,
}

pub struct SimpleBS1DModel {}
