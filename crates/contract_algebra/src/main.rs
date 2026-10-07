use contract_algebra::algebra_ast::{algebra_dsl::*, algebra_printer::*};

fn main() {
    let maturity = 2.;
    let strike = 100.;
    let ul_name = String::from("Toto");
    let currency = String::from("EUR");

    let ul = fixing(ul_name, maturity);

    let call_payoff = max(ul - obs(strike), obs(0.));
    let call_contract = flow(currency, maturity, call_payoff);

    println!("{}", contract_to_string(&call_contract));
}
