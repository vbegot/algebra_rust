// Simple BS 1d model with constant vol, constant rate

use common::currency::Currency;
use mc_algo::model_interface::MCModel;
use rand::{Rng, SeedableRng, rngs::StdRng};
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
pub enum ModelError {}

impl std::fmt::Display for ModelError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for ModelError {}

pub struct SimpleBS1DModel {
    parameters: ModelParameters,
    x: f64,
    rng: StdRng,
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
            rng: StdRng::seed_from_u64(42),
        })
    }

    fn init(
        &mut self,
        as_of: &common::date::Date,
        fix_dates: &[common::date::Date],
        underlyings: &[String],
        flow_infos: &[mc_algo::compiled_contract::compiler::FlowInfos],
    ) -> Result<(), Self::ModelError> {
        panic!("Not Implemented")
    }

    fn init_trajectory(&mut self) {
        panic!("Not Implemented")
    }

    fn simulate(&mut self, idx: usize, from: f64, to: f64) {
        panic!("Not Implemented")
    }

    fn fill_discounts(&self, discounts: &mut [f64]) {
        panic!("Not Implemented")
    }

    fn fill_underlyings(&self, underlyings: &mut [f64]) {
        panic!("Not Implemented")
    }
}
