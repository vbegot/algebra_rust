use common::currency;
use common::date::Date;
use contract_algebra::algebra_ast::algebra_dsl::*;
use contract_algebra::lifecycle::LifecycleContext;
use mc_algo::compiled_contract::compiler::compile;
use mc_algo::model_interface::MCModel;
use mc_algo::pricing_algo::price_mc;
use mc_models::simple_bs1d_model;
use pricer_cli::models::ModelAndMethod;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut bs_model = simple_bs1d_model::SimpleBS1DModel::new(&bs_parameters)?;

    let call_payoff = max(fixing(ul_name.clone(), maturity) - obs(strike), obs(0.));
    let call_contract = flow(currency, maturity, call_payoff);

    let compiled_call_contract = compile(&call_contract, &as_of)?;

    compiled_call_contract.get_compilation_context().show();

    let price = price_mc(&compiled_call_contract, &mut bs_model, 1_000_000)?;

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

    println!(
        "MC Price = {:.4} +/- {:.4} in {:.2}ms, theo price = {theo_call:.4}",
        price.price,
        1.96 * price.stderr,
        price.time
    );

    let mut mc_bs_model = pricer_cli::models::MCModelAndMethod {
        nb_paths: 10_000,
        model: bs_model,
    };

    let strike_date = Date::new(2020, 1, 1).unwrap();
    let call_contract = flow(
        currency,
        maturity,
        max(
            fixing(ul_name.clone(), maturity) - fixing(ul_name.clone(), strike_date),
            obs(0.),
        ),
    );

    let mut fixings = BTreeMap::new();
    let mut toto_fixings = BTreeMap::new();
    toto_fixings.insert(strike_date, strike);
    fixings.insert(ul_name.clone(), toto_fixings);
    let lifecycle_context = LifecycleContext::new(&as_of, fixings);
    let results2 = pricer_cli::models::price(&mut mc_bs_model, &call_contract, &lifecycle_context)?;
    let price2 = mc_bs_model.get_price(&results2);

    println!("Price 2 = {:.4}", price2);

    Ok(())
}
