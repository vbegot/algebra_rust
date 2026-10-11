use super::*;
use common::currency::EUR;
use contract_algebra::algebra_ast::algebra_dsl::*;

fn date(year: u32) -> Date {
    Date::new(year, 1, 1).unwrap()
}

fn spot(name: &str, year: u32) -> Observable {
    fixing(name.into(), date(year))
}

#[test]
fn compilation_rejects_payment_before_condition_and_restores_current_date() {
    let contract = if_contract(
        spot("stock", 2027).gt(obs(100.0)),
        flow(EUR, date(2026), obs(10.0)),
        Contract::All(vec![]),
    );
    let mut compilation = CompilationCtx::new(&contract, &date(2024));
    assert!(matches!(contract.compile(&mut compilation),
        Err(CompilationError::NonMeasurableFlow { fix_date, pay_date })
        if fix_date == date(2027) && pay_date == date(2026)));
    // Only check temporal-state restoration; registration is not transactional.
    let neighbor = flow(EUR, date(2025), obs(3.0));
    neighbor.compile(&mut compilation).unwrap();
    assert_eq!(
        compilation.get_flow_infos().last().unwrap().fix_date,
        date(2024)
    );
}

#[test]
fn compilation_rejects_fixing_missing_from_context_date_grid() {
    let mut compilation = CompilationCtx::new(&Contract::All(vec![]), &date(2024));
    let contract = flow(EUR, date(2028), spot("stock", 2027));
    assert!(matches!(
        contract.compile(&mut compilation),
        Err(CompilationError::BadContext)
    ));
}
