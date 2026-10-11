use super::*;
use crate::algebra_ast::{BinaryOperator, UnaryOperator};

#[test]
fn constant_is_resolved() {
    let (residual, value) = Observable::Constant(42.0).manage(&context()).unwrap();
    assert_constant(&residual, 42.0);
    assert_eq!(value, Some(42.0));
}

#[test]
fn past_and_today_fixings_are_resolved() {
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 100.0);
    add_fixing(&mut ctx, "spot", 2026, 120.0);
    for (year, expected) in [(2025, 100.0), (2026, 120.0)] {
        let (residual, value) = fixing("spot", year).manage(&ctx).unwrap();
        assert_constant(&residual, expected);
        assert_eq!(value, Some(expected));
    }
}

#[test]
fn future_fixing_remains_unknown_even_if_present_in_context() {
    for has_fixing in [false, true] {
        let mut ctx = context();
        if has_fixing {
            add_fixing(&mut ctx, "spot", 2027, 150.0);
        }
        let (residual, value) = fixing("spot", 2027).manage(&ctx).unwrap();
        assert_fixing(&residual, "spot", 2027);
        assert_eq!(value, None);
    }
}

#[test]
fn historical_fixing_errors_identify_underlying_and_date() {
    for year in [2025, 2026] {
        assert!(matches!(fixing("missing", year).manage(&context()),
            Err(LifecycleError::NoFixings(name)) if name == "missing"));
        let mut ctx = context();
        add_fixing(&mut ctx, "spot", 2024, 90.0);
        assert!(matches!(fixing("spot", year).manage(&ctx),
            Err(LifecycleError::MissingFixing(name, day)) if name == "spot" && day == date(year)));
    }
}

#[test]
fn binary_operators_fold_known_operands() {
    for (op, expected) in [
        (BinaryOperator::Plus, 6.0),
        (BinaryOperator::Minus, 2.0),
        (BinaryOperator::Times, 8.0),
        (BinaryOperator::Div, 2.0),
        (BinaryOperator::Max, 4.0),
        (BinaryOperator::Min, 2.0),
        (BinaryOperator::Pow, 16.0),
    ] {
        let input = Observable::BinopObservable {
            left: Box::new(fixing("spot", 2025)),
            op,
            right: Box::new(Observable::Constant(2.0)),
        };
        let mut ctx = context();
        add_fixing(&mut ctx, "spot", 2025, 4.0);
        let (residual, value) = input.manage(&ctx).unwrap();
        assert_constant(&residual, expected);
        assert_eq!(value, Some(expected), "op={op:?}");
    }
}

#[test]
fn partial_expression_preserves_future_fixing_and_folds_known_subtree() {
    let known = Observable::BinopObservable {
        left: Box::new(fixing("spot", 2025)),
        op: BinaryOperator::Times,
        right: Box::new(Observable::Constant(2.0)),
    };
    let input = Observable::BinopObservable {
        left: Box::new(fixing("spot", 2027)),
        op: BinaryOperator::Plus,
        right: Box::new(known),
    };
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 4.0);
    let (residual, value) = input.manage(&ctx).unwrap();
    assert_eq!(value, None);
    match residual {
        Observable::BinopObservable {
            left,
            op: BinaryOperator::Plus,
            right,
        } => {
            assert_fixing(&left, "spot", 2027);
            assert_constant(&right, 8.0);
        }
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn unary_operators_fold_known_operands() {
    for (op, input, expected) in [
        (UnaryOperator::Neg, 4.0, -4.0),
        (UnaryOperator::Log, 1.0, 0.0),
        (UnaryOperator::Exp, 0.0, 1.0),
        (UnaryOperator::Sqrt, 4.0, 2.0),
        (UnaryOperator::Sq, 4.0, 16.0),
        (UnaryOperator::Abs, -4.0, 4.0),
    ] {
        let input = Observable::UnopObservable {
            op,
            obs: Box::new(Observable::Constant(input)),
        };
        let (residual, value) = input.manage(&context()).unwrap();
        assert_constant(&residual, expected);
        assert_eq!(value, Some(expected), "op={op:?}");
    }
}

#[test]
fn unary_operator_preserves_unknown_operand() {
    let input = Observable::UnopObservable {
        op: UnaryOperator::Neg,
        obs: Box::new(fixing("spot", 2027)),
    };
    let (residual, value) = input.manage(&context()).unwrap();
    assert_eq!(value, None);
    match residual {
        Observable::UnopObservable {
            op: UnaryOperator::Neg,
            obs,
        } => assert_fixing(&obs, "spot", 2027),
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn arithmetic_propagates_required_fixing_errors() {
    for missing_on_left in [false, true] {
        let (left, right) = if missing_on_left {
            (fixing("missing", 2025), Observable::Constant(1.0))
        } else {
            (Observable::Constant(1.0), fixing("missing", 2025))
        };
        let input = Observable::BinopObservable {
            left: Box::new(left),
            op: BinaryOperator::Plus,
            right: Box::new(right),
        };
        assert!(matches!(
            input.manage(&context()),
            Err(LifecycleError::NoFixings(_))
        ));
    }
    let input = Observable::UnopObservable {
        op: UnaryOperator::Neg,
        obs: Box::new(fixing("missing", 2025)),
    };
    assert!(matches!(
        input.manage(&context()),
        Err(LifecycleError::NoFixings(_))
    ));
}

#[test]
fn resolved_if_selects_only_relevant_branch() {
    for state in [State::True, State::False] {
        let (yes, no) = if matches!(state, State::True) {
            (Observable::Constant(42.0), fixing("missing", 2025))
        } else {
            (fixing("missing", 2025), Observable::Constant(42.0))
        };
        let input = Observable::IfObservable {
            condition: Box::new(condition(state)),
            true_observable: Box::new(yes),
            false_observable: Box::new(no),
        };
        let (residual, value) = input.manage(&context()).unwrap();
        assert_constant(&residual, 42.0);
        assert_eq!(value, Some(42.0));
    }
}

#[test]
fn unresolved_if_manages_both_branches_and_retains_condition() {
    let input = Observable::IfObservable {
        condition: Box::new(condition(State::Unknown)),
        true_observable: Box::new(fixing("spot", 2025)),
        false_observable: Box::new(fixing("spot", 2027)),
    };
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 100.0);
    let (residual, value) = input.manage(&ctx).unwrap();
    assert_eq!(value, None);
    match residual {
        Observable::IfObservable {
            condition,
            true_observable,
            false_observable,
        } => {
            assert_eq!(condition.manage(&ctx).unwrap().1, None);
            assert_constant(&true_observable, 100.0);
            assert_fixing(&false_observable, "spot", 2027);
        }
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn if_propagates_errors_in_condition_or_required_branches() {
    for (state, yes, no) in [
        (
            State::MissingFixing,
            Observable::Constant(1.0),
            Observable::Constant(2.0),
        ),
        (
            State::True,
            fixing("missing", 2025),
            Observable::Constant(2.0),
        ),
        (
            State::False,
            Observable::Constant(1.0),
            fixing("missing", 2025),
        ),
        (
            State::Unknown,
            fixing("missing", 2025),
            Observable::Constant(2.0),
        ),
        (
            State::Unknown,
            Observable::Constant(1.0),
            fixing("missing", 2025),
        ),
    ] {
        let input = Observable::IfObservable {
            condition: Box::new(condition(state)),
            true_observable: Box::new(yes),
            false_observable: Box::new(no),
        };
        assert!(matches!(
            input.manage(&context()),
            Err(LifecycleError::NoFixings(_))
        ));
    }
}
