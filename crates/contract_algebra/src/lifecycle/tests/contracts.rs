use super::*;

fn flow(year: u32, currency: &'static str, amount: Observable) -> Contract {
    Contract::Flow {
        currency: Currency::new(currency),
        date: date(year),
        amount: Box::new(amount),
    }
}

fn empty(contract: &Contract) {
    assert!(
        matches!(contract, Contract::All(items) if items.is_empty()),
        "expected empty contract: {contract:?}"
    );
}

fn assert_flow(contract: &Contract, year: u32, expected_currency: &str, expected_amount: f64) {
    match contract {
        Contract::Flow {
            currency,
            date: pay_date,
            amount,
        } => {
            assert_eq!(*pay_date, date(year));
            assert_eq!(currency.to_string(), expected_currency);
            assert_constant(amount, expected_amount);
        }
        other => panic!("expected flow: {other:?}"),
    }
}

fn conditional(state: State, yes: Contract, no: Contract) -> Contract {
    Contract::IfContract {
        condition: Box::new(condition(state)),
        true_contract: Box::new(yes),
        false_contract: Box::new(no),
    }
}

#[test]
fn past_flow_is_extracted_with_date_currency_and_signed_amount() {
    for amount in [100.0, -100.0] {
        let result = flow(2025, "USD", Observable::Constant(amount))
            .manage(&context())
            .unwrap();
        empty(&result.managed_contract);
        assert_eq!(result.paid_flows.len(), 1);
        let paid = &result.paid_flows[0];
        assert_eq!(paid.pay_date, date(2025));
        assert_eq!(paid.currency.to_string(), "USD");
        assert_eq!(paid.amount, amount);
        assert!(
            result
                .managed_contract
                .manage(&context())
                .unwrap()
                .paid_flows
                .is_empty()
        );
    }
}

#[test]
fn today_and_future_known_flows_remain_unpaid() {
    for year in [2026, 2027] {
        let result = flow(year, "EUR", Observable::Constant(100.0))
            .manage(&context())
            .unwrap();
        assert!(result.paid_flows.is_empty());
        assert_flow(&result.managed_contract, year, "EUR", 100.0);
    }
}

#[test]
fn zero_flows_are_removed_for_any_payment_date() {
    for year in [2025, 2026, 2027] {
        let mut ctx = context();
        add_fixing(&mut ctx, "spot", 2025, 0.0);
        for amount in [
            Observable::Constant(0.0),
            Observable::Constant(-0.0),
            fixing("spot", 2025),
        ] {
            let result = flow(year, "EUR", amount).manage(&ctx).unwrap();
            assert!(result.paid_flows.is_empty());
            empty(&result.managed_contract);
        }
    }
}

#[test]
fn future_flow_keeps_unknown_amount_and_resolves_known_amount() {
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 100.0);
    let result = flow(2028, "EUR", fixing("spot", 2025))
        .manage(&ctx)
        .unwrap();
    assert!(result.paid_flows.is_empty());
    assert_flow(&result.managed_contract, 2028, "EUR", 100.0);
    let result = flow(2028, "EUR", fixing("spot", 2027))
        .manage(&ctx)
        .unwrap();
    assert!(result.paid_flows.is_empty());
    match result.managed_contract {
        Contract::Flow { amount, .. } => assert_fixing(&amount, "spot", 2027),
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn missing_historical_fixing_in_flow_is_propagated() {
    for year in [2025, 2027] {
        assert!(
            matches!(flow(year, "EUR", fixing("missing", 2025)).manage(&context()), Err(LifecycleError::NoFixings(name)) if name == "missing")
        );
    }
}

#[test]
fn all_collects_paid_flows_and_removes_empty_residuals() {
    let input = Contract::All(vec![
        flow(2025, "EUR", Observable::Constant(10.0)),
        Contract::All(vec![flow(2024, "USD", Observable::Constant(-20.0))]),
        flow(2027, "GBP", Observable::Constant(30.0)),
        flow(2027, "EUR", Observable::Constant(0.0)),
        Contract::All(vec![]),
    ]);
    let result = input.manage(&context()).unwrap();
    assert_eq!(result.paid_flows.len(), 2);
    for (paid, year, currency, amount) in [
        (&result.paid_flows[0], 2025, "EUR", 10.0),
        (&result.paid_flows[1], 2024, "USD", -20.0),
    ] {
        assert_eq!(paid.pay_date, date(year));
        assert_eq!(paid.currency.to_string(), currency);
        assert_eq!(paid.amount, amount);
    }
    match result.managed_contract {
        Contract::All(items) => {
            assert_eq!(items.len(), 1);
            assert_flow(&items[0], 2027, "GBP", 30.0);
        }
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn empty_and_fully_paid_all_leave_empty_contract() {
    for input in [
        Contract::All(vec![]),
        Contract::All(vec![flow(2025, "EUR", Observable::Constant(10.0))]),
    ] {
        let result = input.manage(&context()).unwrap();
        empty(&result.managed_contract);
    }
}

#[test]
fn all_propagates_errors_from_children() {
    let input = Contract::All(vec![
        flow(2027, "EUR", Observable::Constant(10.0)),
        flow(2027, "EUR", fixing("missing", 2025)),
    ]);
    assert!(matches!(
        input.manage(&context()),
        Err(LifecycleError::NoFixings(_))
    ));
}

#[test]
fn resolved_if_selects_branch_and_ignores_unselected_error() {
    for state in [State::True, State::False] {
        let selected = flow(2025, "EUR", Observable::Constant(42.0));
        let discarded = flow(2027, "USD", fixing("missing", 2025));
        let (yes, no) = if matches!(state, State::True) {
            (selected, discarded)
        } else {
            (discarded, selected)
        };
        let result = conditional(state, yes, no).manage(&context()).unwrap();
        empty(&result.managed_contract);
        assert_eq!(result.paid_flows.len(), 1);
        assert_eq!(result.paid_flows[0].amount, 42.0);
        assert_eq!(result.paid_flows[0].currency.to_string(), "EUR");
    }
}

#[test]
fn unresolved_if_preserves_future_branches_without_paid_flows() {
    let input = conditional(
        State::Unknown,
        flow(2028, "EUR", fixing("spot", 2025)),
        flow(2028, "USD", Observable::Constant(20.0)),
    );
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 10.0);
    let result = input.manage(&ctx).unwrap();
    assert!(result.paid_flows.is_empty());
    match result.managed_contract {
        Contract::IfContract {
            condition,
            true_contract,
            false_contract,
        } => {
            assert_eq!(condition.manage(&ctx).unwrap().1, None);
            assert_flow(&true_contract, 2028, "EUR", 10.0);
            assert_flow(&false_contract, 2028, "USD", 20.0);
        }
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn unresolved_if_rejects_paid_flows_in_either_branch() {
    for paid_on_left in [false, true] {
        let paid = Contract::All(vec![flow(2025, "EUR", Observable::Constant(10.0))]);
        let future = flow(2028, "EUR", Observable::Constant(20.0));
        let (yes, no) = if paid_on_left {
            (paid, future)
        } else {
            (future, paid)
        };
        assert!(matches!(
            conditional(State::Unknown, yes, no).manage(&context()),
            Err(LifecycleError::InvalidContract)
        ));
    }
}

#[test]
fn conditional_contract_propagates_required_errors() {
    for state in [
        State::MissingFixing,
        State::True,
        State::False,
        State::Unknown,
    ] {
        let input = conditional(
            state,
            flow(2028, "EUR", fixing("missing", 2025)),
            flow(2028, "EUR", fixing("missing", 2025)),
        );
        assert!(matches!(
            input.manage(&context()),
            Err(LifecycleError::NoFixings(_))
        ));
    }
}

#[test]
fn past_payment_with_future_amount_is_invalid() {
    // A payment cannot depend on a fixing occurring after the payment date.
    let input = flow(2025, "EUR", fixing("spot", 2027));
    assert!(matches!(
        input.manage(&context()),
        Err(LifecycleError::InvalidContract)
    ));
}

#[test]
fn unresolved_if_rejects_past_payment_even_when_amount_is_unknown() {
    let input = conditional(
        State::Unknown,
        flow(2025, "EUR", fixing("spot", 2027)),
        Contract::All(vec![]),
    );
    assert!(matches!(
        input.manage(&context()),
        Err(LifecycleError::InvalidContract)
    ));
}
