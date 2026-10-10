use super::*;

mod conditions;
mod contracts;
mod observables;

fn date(year: u32) -> Date {
    Date::new(year, 1, 1).unwrap()
}

fn context() -> LifecycleContext {
    LifecycleContext {
        as_of: date(2026),
        fixings: BTreeMap::new(),
    }
}

fn fixing(name: &str, year: u32) -> Observable {
    Observable::Fixing {
        name: name.into(),
        fixing_date: date(year),
    }
}

fn add_fixing(ctx: &mut LifecycleContext, name: &str, year: u32, value: f64) {
    ctx.fixings
        .entry(name.into())
        .or_default()
        .insert(date(year), value);
}

fn assert_constant(obs: &Observable, expected: f64) {
    match obs {
        Observable::Constant(value) => assert_eq!(*value, expected),
        other => panic!("expected Constant({expected}), got {other:?}"),
    }
}

fn assert_fixing(obs: &Observable, name: &str, year: u32) {
    assert!(
        matches!(obs, Observable::Fixing { name: actual, fixing_date }
        if actual == name && *fixing_date == date(year)),
        "unexpected observable: {obs:?}"
    );
}

#[derive(Clone, Copy, Debug)]
enum State {
    False,
    True,
    Unknown,
    MissingFixing,
}

fn condition(state: State) -> ObsCondition {
    let left = match state {
        State::False => Observable::Constant(0.0),
        State::True => Observable::Constant(1.0),
        State::Unknown | State::MissingFixing => Observable::Fixing {
            name: "test".into(),
            fixing_date: Date::new(
                if matches!(state, State::Unknown) {
                    2027
                } else {
                    2025
                },
                1,
                1,
            )
            .unwrap(),
        },
    };
    ObsCondition::SimpleCondition {
        left: Box::new(left),
        comp: Higher,
        right: Box::new(Observable::Constant(0.0)),
    }
}

#[test]
fn all_boolean_resolution_and_error_combinations() {
    let ctx = LifecycleContext {
        as_of: Date::new(2026, 1, 1).unwrap(),
        fixings: BTreeMap::new(),
    };
    let states = [
        State::False,
        State::True,
        State::Unknown,
        State::MissingFixing,
    ];

    // Rows and columns follow `states`: false, true, unknown, missing fixing.
    // These tables specify the expected semantics independently of operator evaluation.
    let and_results = [
        [
            Ok(Some(false)),
            Ok(Some(false)),
            Ok(Some(false)),
            Ok(Some(false)),
        ],
        [Ok(Some(false)), Ok(Some(true)), Ok(None), Err(())],
        [Ok(Some(false)), Ok(None), Ok(None), Err(())],
        [Ok(Some(false)), Err(()), Err(()), Err(())],
    ];
    let or_results = [
        [Ok(Some(false)), Ok(Some(true)), Ok(None), Err(())],
        [
            Ok(Some(true)),
            Ok(Some(true)),
            Ok(Some(true)),
            Ok(Some(true)),
        ],
        [Ok(None), Ok(Some(true)), Ok(None), Err(())],
        [Err(()), Ok(Some(true)), Err(()), Err(())],
    ];

    for (op, expected_results) in [
        (BinaryCondOperator::And, and_results),
        (BinaryCondOperator::Or, or_results),
    ] {
        for (left_index, left) in states.into_iter().enumerate() {
            for (right_index, right) in states.into_iter().enumerate() {
                let input = ObsCondition::BinopCondition {
                    left: Box::new(condition(left)),
                    op,
                    right: Box::new(condition(right)),
                };
                let actual = input.manage(&ctx);
                assert_eq!(
                    actual.as_ref().map(|(_, value)| *value).map_err(|_| ()),
                    expected_results[left_index][right_index],
                    "left={left:?}, op={op:?}, right={right:?}"
                );

                match actual {
                    Ok((residual, value)) => {
                        let (_, second_value) = residual.manage(&ctx).unwrap();
                        assert_eq!(
                            second_value, value,
                            "residual must remain manageable: left={left:?}, op={op:?}, right={right:?}"
                        );
                    }
                    Err(error) => assert!(
                        matches!(error, LifecycleError::NoFixings(ref name) if name == "test"),
                        "unexpected error: {error:?}"
                    ),
                }
            }
        }
    }
}
