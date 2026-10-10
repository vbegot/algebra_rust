use super::*;
use crate::algebra_ast::Comparison;

#[test]
fn comparisons_resolve_known_values_including_equal_boundaries() {
    for (comp, right, expected) in [
        (Comparison::Lower, 5.0, true),
        (Comparison::Lower, 4.0, false),
        (Comparison::LowertOrEqual, 4.0, true),
        (Comparison::LowertOrEqual, 3.0, false),
        (Comparison::Higher, 3.0, true),
        (Comparison::Higher, 4.0, false),
        (Comparison::HigherOrEqual, 4.0, true),
        (Comparison::HigherOrEqual, 5.0, false),
    ] {
        let input = ObsCondition::SimpleCondition {
            left: Box::new(fixing("spot", 2025)),
            comp,
            right: Box::new(Observable::Constant(right)),
        };
        let mut ctx = context();
        add_fixing(&mut ctx, "spot", 2025, 4.0);
        let (residual, value) = input.manage(&ctx).unwrap();
        assert_eq!(value, Some(expected), "comparison={comp:?}");
        match residual {
            ObsCondition::SimpleCondition {
                left,
                right: residual_right,
                ..
            } => {
                assert_constant(&left, 4.0);
                assert_constant(&residual_right, right);
            }
            other => panic!("unexpected residual: {other:?}"),
        }
    }
}

#[test]
fn comparison_with_future_fixing_remains_unknown() {
    let input = fixing("spot", 2027).gt(fixing("spot", 2025));
    let mut ctx = context();
    add_fixing(&mut ctx, "spot", 2025, 4.0);
    let (residual, value) = input.manage(&ctx).unwrap();
    assert_eq!(value, None);
    match residual {
        ObsCondition::SimpleCondition { left, right, .. } => {
            assert_fixing(&left, "spot", 2027);
            assert_constant(&right, 4.0);
        }
        other => panic!("unexpected residual: {other:?}"),
    }
}

#[test]
fn not_handles_known_unknown_and_error_states() {
    for (state, expected) in [
        (State::True, Some(false)),
        (State::False, Some(true)),
        (State::Unknown, None),
    ] {
        let input = ObsCondition::Not(Box::new(condition(state)));
        let (residual, value) = input.manage(&context()).unwrap();
        assert_eq!(value, expected);
        assert_eq!(residual.manage(&context()).unwrap().1, expected);
    }
    let input = ObsCondition::Not(Box::new(condition(State::MissingFixing)));
    assert!(matches!(
        input.manage(&context()),
        Err(LifecycleError::NoFixings(_))
    ));
}

#[test]
fn boolean_identity_keeps_the_correct_unknown_condition() {
    for (op, identity) in [
        (BinaryCondOperator::And, State::True),
        (BinaryCondOperator::Or, State::False),
    ] {
        for unknown_on_left in [false, true] {
            let (left, right) = if unknown_on_left {
                (condition(State::Unknown), condition(identity))
            } else {
                (condition(identity), condition(State::Unknown))
            };
            let input = ObsCondition::BinopCondition {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
            let (residual, value) = input.manage(&context()).unwrap();
            assert_eq!(value, None);
            match residual {
                ObsCondition::SimpleCondition { left, .. } => assert_fixing(&left, "test", 2027),
                other => panic!("identity should eliminate the boolean operator: {other:?}"),
            }
        }
    }
}

#[test]
fn nested_boolean_resolution_drops_unnecessary_missing_fixings() {
    let inner = ObsCondition::BinopCondition {
        left: Box::new(condition(State::False)),
        op: BinaryCondOperator::And,
        right: Box::new(condition(State::MissingFixing)),
    };
    let input = ObsCondition::BinopCondition {
        left: Box::new(ObsCondition::Not(Box::new(inner))),
        op: BinaryCondOperator::Or,
        right: Box::new(condition(State::MissingFixing)),
    };
    assert_eq!(input.manage(&context()).unwrap().1, Some(true));
}
