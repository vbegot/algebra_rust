use common::currency;
use common::date::Date;
use contract_algebra::algebra_ast::{algebra_dsl::*, algebra_printer::*};
use mc_algo::compiled_contract::compiler::{self, Compilable};

fn main() {
    let maturity = Date::new(2025, 1, 1).unwrap();
    let strike = 100.;
    let ul_name = String::from("Toto");
    let currency = currency::EUR;

    let ul = fixing(ul_name, maturity);

    let call_payoff = max(ul - obs(strike), obs(0.));
    let call_contract = flow(currency, maturity, call_payoff);

    let mut compilation_context = compiler::CompilationCtx::new(&call_contract);
    let _ = call_contract.compile(&mut compilation_context);

    println!("Call contract:\n{}", contract_to_string(&call_contract));
}
