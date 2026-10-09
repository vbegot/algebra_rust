use crate::{
    algebra_ast::{Contract, ObsCondition, Observable},
    lifecycle::LifecycleError::{MissingFixing, NoFixings},
};
use common::{currency::Currency, date::Date};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display};

#[allow(dead_code)]
struct PaidFlow {
    pay_date: Date,
    currency: Currency,
    amount: f64,
}

#[allow(dead_code)]
pub struct LifecycleResult {
    paid_flows: Vec<PaidFlow>,
    managed_contract: Contract,
}

pub struct LifecycleContext {
    as_of: Date,
    fixings: BTreeMap<String, BTreeMap<Date, f64>>,
}

#[derive(Debug)]
pub enum LifecycleError {
    NoFixings(String),           // No fixing at all for the given underlying
    MissingFixing(String, Date), // Missing the fixing date for the underlying
    FunctionalError(String),
}

impl Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFixings(name) => write!(f, "No fixings for {name}"),
            Self::MissingFixing(name, date) => {
                write!(f, "Missing fixing for {name} on {}", date.to_string())
            }
            Self::FunctionalError(message) => f.write_str(message),
        }
    }
}

impl Error for LifecycleError {}

pub trait Managable {
    type Output;

    fn manage(&self, ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError>;
}

fn resolved_obs(x: f64) -> Result<(Observable, Option<f64>), LifecycleError> {
    Ok((Observable::Constant(x), Some(x)))
}

impl Managable for Observable {
    type Output = (Observable, Option<f64>);

    fn manage(&self, ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError> {
        match self {
            Self::Constant(x) => Ok((self.clone(), Some(*x))),
            Self::Fixing { name, fixing_date } => {
                if fixing_date <= &ctx.as_of {
                    let ul_fixings = ctx.fixings.get(name).ok_or(NoFixings(name.clone()))?;
                    let fixing = ul_fixings
                        .get(fixing_date)
                        .ok_or(MissingFixing(name.clone(), *fixing_date))?;
                    resolved_obs(*fixing)
                } else {
                    Ok((self.clone(), None))
                }
            }
            Self::BinopObservable { left, op, right } => {
                let (left_obs, left_resolved) = left.manage(ctx)?;
                let (right_obs, right_resolved) = right.manage(ctx)?;
                match (left_resolved, right_resolved) {
                    (Some(explicit_left), Some(explicit_right)) => {
                        resolved_obs(op.eval(explicit_left, explicit_right))
                    }
                    _ => {
                        let obs = Observable::BinopObservable {
                            left: Box::new(left_obs),
                            op: *op,
                            right: Box::new(right_obs),
                        };
                        Ok((obs, None))
                    }
                }
            }
            Self::UnopObservable { op, obs } => {
                let (managed_obs, resolved) = obs.manage(ctx)?;
                match resolved {
                    Some(x) => resolved_obs(op.eval(x)),
                    None => Ok((
                        Observable::UnopObservable {
                            op: *op,
                            obs: Box::new(managed_obs),
                        },
                        None,
                    )),
                }
            }
            Self::IfObservable {
                condition,
                true_observable,
                false_observable,
            } => {
                let (cond, cond_resolved) = condition.manage(ctx)?;
                match cond_resolved {
                    Some(true) => true_observable.manage(ctx),
                    Some(false) => false_observable.manage(ctx),
                    None => {
                        let (true_obs, _) = true_observable.manage(ctx)?;
                        let (false_obs, _) = false_observable.manage(ctx)?;
                        let obs = Observable::IfObservable {
                            condition: Box::new(cond),
                            true_observable: Box::new(true_obs),
                            false_observable: Box::new(false_obs),
                        };
                        Ok((obs, None))
                    }
                }
            }
        }
    }
}

impl Managable for ObsCondition {
    type Output = (ObsCondition, Option<bool>);

    fn manage(&self, _ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError> {
        Err(LifecycleError::FunctionalError(String::from("TODO")))
    }
}

impl Managable for Contract {
    type Output = LifecycleResult;

    fn manage(&self, _ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError> {
        Err(LifecycleError::FunctionalError(String::from("TODO")))
    }
}
