use common::{currency::EUR, date::Date, matrix::Matrix};
use contract_algebra::algebra_ast::{Contract, Observable, UnaryOperator, algebra_dsl::*};
use mc_algo::compiled_contract::{
    EvalContext,
    compiler::{CompilationCtx, CompilationError, compile},
};

fn date(year: u32) -> Date {
    Date::new(year, 1, 1).unwrap()
}

fn spot(name: &str, year: u32) -> Observable {
    fixing(name.into(), date(year))
}

fn trajectory(ctx: &CompilationCtx, values: &[(u32, &str, f64)]) -> Matrix {
    let dates = ctx.get_fix_dates();
    let underlyings = ctx.get_underlyings();
    let mut matrix = Matrix::new(dates.len(), underlyings.len());
    for (row, fixing_date) in dates.iter().enumerate() {
        for (column, underlying) in underlyings.iter().enumerate() {
            let value = values
                .iter()
                .find(|(year, name, _)| date(*year) == *fixing_date && *name == underlying)
                .unwrap_or_else(|| {
                    panic!(
                        "missing deterministic value for {underlying} at {}",
                        fixing_date.to_string()
                    )
                })
                .2;
            matrix[(row, column)] = value;
        }
    }
    matrix
}

fn assert_value(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}

#[test]
fn call_compiles_and_evaluates_below_at_and_above_strike() {
    let contract = flow(
        EUR,
        date(2025),
        max(spot("stock", 2025) - obs(100.0), obs(0.0)),
    );
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();

    assert_eq!(compilation.get_as_of(), date(2024));
    assert_eq!(compilation.get_fix_dates(), vec![date(2025)]);
    assert_eq!(compilation.get_underlyings(), vec!["stock"]);
    let infos = compilation.get_flow_infos();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].fix_date, date(2025));
    assert_eq!(infos[0].pay_date, date(2025));
    assert_eq!(infos[0].currency.to_string(), "EUR");

    for (stock, expected) in [(80.0, 0.0), (100.0, 0.0), (120.0, 18.0)] {
        let mut path = trajectory(compilation, &[(2025, "stock", stock)]);
        let mut discounts = [0.9];
        let mut ctx = EvalContext::new(&mut discounts, &mut path);
        // The same compiled object is reused with independent deterministic contexts.
        assert_value(compiled.eval(&mut ctx), expected);
    }
}

#[test]
fn fixed_flow_evaluates_without_any_fixing_or_underlying() {
    let contract = flow(EUR, date(2028), obs(-25.0));
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();
    assert!(compilation.get_fix_dates().is_empty());
    assert!(compilation.get_underlyings().is_empty());
    assert_eq!(compilation.get_flow_infos()[0].fix_date, date(2024));
    let mut path = Matrix::new(0, 0);
    let mut discounts = [0.8];
    assert_value(
        compiled.eval(&mut EvalContext::new(&mut discounts, &mut path)),
        -20.0,
    );
}

#[test]
fn portfolio_uses_sorted_dates_and_registration_order_for_underlyings() {
    let contract = all(vec![
        flow(EUR, date(2028), spot("Z", 2027) - spot("A", 2025)),
        flow(EUR, date(2026), spot("Z", 2025) / obs(2.0)),
    ]);
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();
    assert_eq!(compilation.get_fix_dates(), vec![date(2025), date(2027)]);
    // Alphabetical map iteration must not reorder assigned column indices.
    assert_eq!(compilation.get_underlyings(), vec!["Z", "A"]);
    assert_eq!(compilation.get_flow_infos().len(), 2);
    assert_eq!(compilation.get_flow_infos()[0].fix_date, date(2027));
    assert_eq!(compilation.get_flow_infos()[1].fix_date, date(2025));
    let mut path = trajectory(
        compilation,
        &[
            (2025, "Z", 8.0),
            (2025, "A", 4.0),
            (2027, "Z", 30.0),
            (2027, "A", 999.0),
        ],
    );
    let mut discounts = [0.8, 0.95];
    // (30 - 4) * 0.8 + (8 / 2) * 0.95 = 24.6.
    assert_value(
        compiled.eval(&mut EvalContext::new(&mut discounts, &mut path)),
        24.6,
    );
}

#[test]
fn conditional_observable_compiles_boolean_and_numeric_expressions() {
    let condition = (spot("stock", 2025).geq(obs(100.0)) & not(spot("stock", 2025).gt(obs(130.0))))
        | spot("stock", 2025).lt(obs(50.0));
    let positive_amount = Observable::UnopObservable {
        op: UnaryOperator::Sqrt,
        obs: Box::new(pow(spot("stock", 2025) - obs(100.0), obs(2.0))),
    };
    let amount = if_observable(condition, min(positive_amount, obs(25.0)), obs(-5.0));
    let contract = flow(EUR, date(2026), amount);
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();
    assert_eq!(compilation.get_fix_dates(), vec![date(2025)]);
    assert_eq!(compilation.get_underlyings(), vec!["stock"]);
    assert_eq!(compilation.get_flow_infos()[0].fix_date, date(2025));
    for (stock, expected) in [(120.0, 10.0), (100.0, 0.0), (140.0, -2.5), (40.0, 12.5)] {
        let mut path = trajectory(compilation, &[(2025, "stock", stock)]);
        let mut discounts = [0.5];
        assert_value(
            compiled.eval(&mut EvalContext::new(&mut discounts, &mut path)),
            expected,
        );
    }
}

#[test]
fn nested_contract_conditions_set_flow_dates_and_restore_neighbor_date() {
    let inner = if_contract(
        spot("stock", 2026).gt(obs(50.0)),
        flow(EUR, date(2029), spot("stock", 2025)),
        flow(EUR, date(2029), obs(7.0)),
    );
    let outer = if_contract(
        spot("stock", 2027).gt(obs(100.0)),
        inner,
        flow(EUR, date(2029), spot("stock", 2028)),
    );
    let contract = all(vec![outer, flow(EUR, date(2029), obs(3.0))]);
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();
    let infos = compilation.get_flow_infos();
    assert_eq!(infos.len(), 4);
    assert_eq!(
        infos.iter().map(|flow| flow.fix_date).collect::<Vec<_>>(),
        vec![date(2027), date(2027), date(2028), date(2024)]
    );
    for (inner_value, outer_value, expected) in [
        (80.0, 120.0, 22.4), // Inner true: 40 * 0.5 + 3 * 0.8.
        (20.0, 120.0, 5.2),  // Inner false: 7 * 0.4 + 3 * 0.8.
        (80.0, 90.0, 20.4),  // Outer false: 60 * 0.3 + 3 * 0.8.
    ] {
        let mut path = trajectory(
            compilation,
            &[
                (2025, "stock", 40.0),
                (2026, "stock", inner_value),
                (2027, "stock", outer_value),
                (2028, "stock", 60.0),
            ],
        );
        let mut discounts = [0.5, 0.4, 0.3, 0.8];
        assert_value(
            compiled.eval(&mut EvalContext::new(&mut discounts, &mut path)),
            expected,
        );
    }
}

#[test]
fn empty_contract_compiles_and_evaluates_to_zero() {
    let contract = Contract::All(vec![]);
    let compiled = compile(&contract, &date(2024)).unwrap();
    let compilation = compiled.get_compilation_context();
    assert!(compilation.get_flow_infos().is_empty());
    let mut path = Matrix::new(0, 0);
    assert_value(
        compiled.eval(&mut EvalContext::new(&mut [], &mut path)),
        0.0,
    );
}

#[test]
fn compilation_rejects_payment_before_amount_fixing() {
    let contract = flow(EUR, date(2026), spot("stock", 2027));
    assert!(matches!(compile(&contract, &date(2024)),
        Err(CompilationError::NonMeasurableFlow { fix_date, pay_date })
        if fix_date == date(2027) && pay_date == date(2026)));
}
