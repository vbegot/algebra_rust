use common::currency;
use common::date::Date;
use contract_algebra::algebra_ast::algebra_dsl::*;
use mc_algo::compiled_contract::compiler::compile;
use mc_algo::model_interface::MCModel;
use mc_algo::pricing_algo::price_mc;
use mc_models::simple_bs1d_model;

#[test]
fn test_call() {
    let as_of = Date::new(2024, 1, 1).unwrap();
    let maturity = Date::new(2025, 1, 1).unwrap();
    let strike = 100.;
    let ul_name = String::from("Toto");
    let currency = currency::EUR;

    let spot = 100.;
    let rate = 0.02;
    let volatility = 0.3;

    let bs_parameters = simple_bs1d_model::ModelParameters {
        underlying: ul_name.clone(),
        spot,
        volatility,
        rate,
        currency,
    };

    let mut bs_model = simple_bs1d_model::SimpleBS1DModel::new(&bs_parameters).unwrap();

    let call_payoff = max(fixing(ul_name, maturity) - obs(strike), obs(0.));
    let call_contract = flow(currency, maturity, call_payoff);

    let compiled_call_contract = compile(&call_contract, &as_of).unwrap();

    let price = price_mc(&compiled_call_contract, &mut bs_model, 100_000).unwrap();

    let dt = (maturity - as_of) as f64 / 365.;
    let zc = (-rate * dt).exp();
    let fwd = spot / zc;

    let theo_call = common::black::black_price(
        fwd,
        zc,
        volatility,
        dt,
        strike,
        common::black::CallPut::Call,
    );

    assert!((price.price - theo_call).abs() < price.stderr)
}
