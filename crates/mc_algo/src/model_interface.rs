use crate::compiled_contract::compiler::FlowInfos;
use common::date::Date;

pub trait MCModel {
    type ModelParameters;
    type ModelError;

    fn new(parameters: &Self::ModelParameters) -> Result<Self, Self::ModelError>
    where
        Self: Sized;

    fn init(
        &mut self,
        as_of: &Date,
        fix_dates: &[Date],
        underlyings: &[String],
        flow_infos: &[FlowInfos],
    ) -> Option<Self::ModelError>;

    fn init_trajectory(&mut self);

    // from, to are dates expressed as year fraction (ACT/365) from as of
    fn simulate(&mut self, idx: usize, from: f64, to: f64);

    fn fill_underlyings(&self, underlyings: &mut [f64]);

    // Fill only the discounts that are known at last to
    fn fill_discounts(&self, discounts: &mut [f64]);
}
