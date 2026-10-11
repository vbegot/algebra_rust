use common::date;
use contract_algebra::algebra_ast;
use contract_algebra::lifecycle::Managable;
use mc_algo::compiled_contract::compiler;
use mc_algo::model_interface;

pub trait ModelOutput {
    type Output;
}

trait ModelAndMethodImpl: ModelOutput {
    fn price_no_manage(
        &mut self,
        contract: &algebra_ast::Contract,
        as_of: &date::Date,
    ) -> Result<Self::Output, Box<dyn std::error::Error>>;

    fn get_price(&self, pricing_output: &Self::Output) -> f64;
}

pub trait ModelAndMethod: ModelOutput {
    fn price(
        &mut self,
        contract: &algebra_ast::Contract,
        lifecycle_context: &contract_algebra::lifecycle::LifecycleContext,
    ) -> Result<Self::Output, Box<dyn std::error::Error>>;

    fn get_price(&self, pricing_output: &Self::Output) -> f64;
}

impl<T: ModelAndMethodImpl> ModelAndMethod for T {
    fn price(
        &mut self,
        contract: &algebra_ast::Contract,
        lifecycle_context: &contract_algebra::lifecycle::LifecycleContext,
    ) -> Result<Self::Output, Box<dyn std::error::Error>> {
        let managed_contract = contract.manage(lifecycle_context)?;
        self.price_no_manage(
            managed_contract.get_managed_contract(),
            lifecycle_context.get_as_of(),
        )
    }

    fn get_price(&self, pricing_output: &Self::Output) -> f64 {
        ModelAndMethodImpl::get_price(self, pricing_output)
    }
}

pub struct MCModelAndMethod<Model: model_interface::MCModel> {
    pub nb_paths: usize,
    pub model: Model,
}

impl<Model> ModelOutput for MCModelAndMethod<Model>
where
    Model: model_interface::MCModel,
    Model::ModelError: 'static,
{
    type Output = mc_algo::pricing_algo::MCResults;
}

impl<Model> ModelAndMethodImpl for MCModelAndMethod<Model>
where
    Model: model_interface::MCModel,
    Model::ModelError: 'static,
{
    fn price_no_manage(
        &mut self,
        contract: &algebra_ast::Contract,
        as_of: &date::Date,
    ) -> Result<Self::Output, Box<dyn std::error::Error>> {
        let compiled_call_contract = compiler::compile(contract, as_of)?;
        Ok(mc_algo::pricing_algo::price_mc(
            &compiled_call_contract,
            &mut self.model,
            self.nb_paths,
        )?)
    }

    fn get_price(&self, pricing_output: &Self::Output) -> f64 {
        pricing_output.price
    }
}

pub fn price<T: ModelAndMethod>(
    model: &mut T,
    contract: &algebra_ast::Contract,
    lifecycle_context: &contract_algebra::lifecycle::LifecycleContext,
) -> Result<T::Output, Box<dyn std::error::Error>> {
    model.price(contract, lifecycle_context)
}
