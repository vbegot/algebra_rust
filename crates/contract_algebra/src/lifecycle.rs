use std::collections::{BTreeMap, HashMap};

use crate::{
    algebra_ast::{Contract, ObsCondition, Observable},
    lifecycle::LifecycleError::{MissingFixing, NoFixings},
};
use common::date::{self, Date};

pub struct LifecycleResult {
    paid_flows: Vec<(date::Date, f64)>,
    managed_contract: Contract,
}

pub struct LifecycleContext {
    as_of: Date,
    fixings: BTreeMap<String, BTreeMap<Date, f64>>,
}

pub enum LifecycleError {
    NoFixings(String),           // No fixing at all for the given underlying
    MissingFixing(String, Date), // Missing the fixing date for the underlying
    FunctionalError(String),
}

pub trait Managable {
    type Output;

    fn manage(&self, ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError>;
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
                    Ok((Observable::Constant(*fixing), Some(*fixing)))
                } else {
                    Ok((self.clone(), None))
                }
            }
            Self::BinopObservable { left, op, right } => {
                let (left_obs, left_resolved) = left.manage(ctx)?;
                let (right_obs, right_resolved) = right.manage(ctx)?;
                let obs = Observable::BinopObservable {
                    left: Box::new(left_obs),
                    op: *op,
                    right: Box::new(right_obs),
                };
                let resolved = match (left_resolved, right_resolved) {
                    (Some(explicit_left), Some(explicit_right)) => {
                        Some(op.eval(explicit_left, explicit_right))
                    }
                    _ => None,
                };
                Ok((obs, resolved))
            }
            Self::UnopObservable { op, obs } => {
                let (managed_obs, resolved) = obs.manage(ctx)?;
                let unop_resolved = resolved.map(|x| op.eval(x));
                Ok((
                    Observable::UnopObservable {
                        op: *op,
                        obs: Box::new(managed_obs),
                    },
                    unop_resolved,
                ))
            }
            Self::IfObservable {
                condition,
                true_observable,
                false_observable,
            } => {
                let (cond, cond_resolved) = condition.manage(ctx)?;
                let managed_true = true_observable.manage(ctx);
                let managed_false = false_observable.manage(ctx);
                match cond_resolved {
                    Some(true) => managed_true,
                    Some(false) => managed_false,
                    None => {
                        let (true_obs, _) = managed_true?;
                        let (false_obs, _) = managed_false?;
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
