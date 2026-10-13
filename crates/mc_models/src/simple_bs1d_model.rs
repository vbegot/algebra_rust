// Simple BS 1d model with constant vol, constant rate

use common::currency::Currency;
use mc_algo::{compiled_contract::compiler::FlowInfos, model_interface::MCModel};
use rand::{SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, StandardNormal};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct ModelParameters {
    pub underlying: String,
    pub spot: f64,
    pub volatility: f64,
    pub rate: f64,
    pub currency: Currency,
}

#[derive(Debug)]
pub enum ModelError {
    TooManyUnderlyings(usize),
    BadUnderlying { expected: String, found: String },
    BadCurrency { expected: Currency, found: Currency },
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyUnderlyings(n) => write!(f, "Expect 1 underlying, found {n} in contract"),
            Self::BadUnderlying { expected, found } => {
                write!(f, "Expect {expected} underlying, found {found} in contract")
            }
            Self::BadCurrency { expected, found } => {
                write!(f, "Expect {expected} currency, found {found} in contract")
            }
        }
    }
}

impl Error for ModelError {}

pub struct SimpleBS1DModel {
    parameters: ModelParameters,
    x: f64,
    rng: StdRng,
    mod_forwards: Vec<f64>, // Contains F(0, t)exp (-sigma²/2 t)
    discounts: Vec<f64>,
    current_idx: usize,
    initialized_discounts: bool,
    zero_underlying: bool,
}

impl MCModel for SimpleBS1DModel {
    type ModelParameters = ModelParameters;
    type ModelError = ModelError;

    fn new(parameters: &Self::ModelParameters) -> Result<Self, Self::ModelError>
    where
        Self: Sized,
    {
        Ok(Self {
            parameters: parameters.clone(),
            x: 0.0,
            rng: StdRng::from_os_rng(),
            mod_forwards: Vec::new(),
            discounts: Vec::new(),
            current_idx: 0,
            initialized_discounts: false,
            zero_underlying: false,
        })
    }

    fn init(
        &mut self,
        as_of: &common::date::Date,
        fix_dates: &[common::date::Date],
        underlyings: &[String],
        flow_infos: &[mc_algo::compiled_contract::compiler::FlowInfos],
    ) -> Result<(), Self::ModelError> {
        let nb_underlyings = underlyings.len();
        if nb_underlyings > 1 {
            return Err(ModelError::TooManyUnderlyings(underlyings.len()));
        }
        self.zero_underlying = nb_underlyings == 0;
        if !self.zero_underlying && underlyings[0] != self.parameters.underlying {
            return Err(ModelError::BadUnderlying {
                expected: self.parameters.underlying.clone(),
                found: underlyings[0].clone(),
            });
        }
        self.x = 0.0;
        self.rng = StdRng::seed_from_u64(42);
        self.current_idx = 0;
        if !self.zero_underlying {
            self.mod_forwards = fix_dates
                .iter()
                .map(|date| {
                    let t = (*date - *as_of) as f64 / 365.0;
                    let vol = self.parameters.volatility;
                    let log = (self.parameters.rate - 0.5 * vol * vol) * t;
                    self.parameters.spot * log.exp()
                })
                .collect();
        }
        self.discounts.resize(flow_infos.len(), 0.0);
        for (
            idx,
            FlowInfos {
                currency, pay_date, ..
            },
        ) in flow_infos.iter().enumerate()
        {
            if *currency != self.parameters.currency {
                return Err(ModelError::BadCurrency {
                    expected: self.parameters.currency,
                    found: *currency,
                });
            }
            let t = (*pay_date - *as_of) as f64 / 365.0;
            self.discounts[idx] = (-self.parameters.rate * t).exp();
        }
        self.initialized_discounts = false;
        Ok(())
    }

    fn init_trajectory(&mut self) {
        self.current_idx = 0;
        self.x = 0.0;
    }

    fn simulate(&mut self, idx: usize, from: f64, to: f64) {
        self.current_idx = idx;
        let z: f64 = StandardNormal.sample(&mut self.rng);
        self.x += self.parameters.volatility * (to - from).sqrt() * z;
    }

    fn fill_discounts(&mut self, discounts: &mut [f64]) {
        if !self.initialized_discounts {
            self.discounts
                .iter()
                .enumerate()
                .for_each(|(idx, &disc)| discounts[idx] = disc);
            self.initialized_discounts = true
        }
    }

    fn fill_underlyings(&mut self, underlyings: &mut [f64]) {
        if !self.zero_underlying {
            underlyings[0] = self.mod_forwards[self.current_idx] * self.x.exp()
        }
    }
}
