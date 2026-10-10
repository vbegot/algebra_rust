use crate::{
    algebra_ast::{
        BinaryCondOperator,
        Comparison::{Higher, Lower},
        Contract, ObsCondition, Observable,
    },
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
    InvalidContract,
}

impl Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFixings(name) => write!(f, "No fixings for {name}"),
            Self::MissingFixing(name, date) => {
                write!(f, "Missing fixing for {name} on {}", date.to_string())
            }
            Self::FunctionalError(message) => f.write_str(message),
            Self::InvalidContract => f.write_str("Invalid initial contract"),
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
                    (Some(x), Some(y)) => resolved_obs(op.eval(x, y)),
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

fn resolved_cond(b: bool) -> Result<(ObsCondition, Option<bool>), LifecycleError> {
    let cond = ObsCondition::SimpleCondition {
        left: Box::new(Observable::Constant(1.0)),
        comp: if b { Higher } else { Lower },
        right: Box::new(Observable::Constant(0.0)),
    };
    Ok((cond, Some(b)))
}

impl Managable for ObsCondition {
    type Output = (ObsCondition, Option<bool>);

    fn manage(&self, ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError> {
        match self {
            ObsCondition::SimpleCondition { left, comp, right } => {
                let (obs_left, explicit_left) = left.manage(ctx)?;
                let (obs_right, explicit_right) = right.manage(ctx)?;
                let cond = ObsCondition::SimpleCondition {
                    left: Box::new(obs_left),
                    comp: *comp,
                    right: Box::new(obs_right),
                };
                let explicit = match (explicit_left, explicit_right) {
                    (Some(x), Some(y)) => Some(comp.eval(x, y)),
                    _ => None,
                };
                Ok((cond, explicit))
            }
            ObsCondition::Not(cond) => {
                let (cond, explicit) = cond.manage(ctx)?;
                let cond = ObsCondition::Not(Box::new(cond));
                let explicit = explicit.map(|x| !x);
                Ok((cond, explicit))
            }
            ObsCondition::BinopCondition { left, op, right } => {
                let managed_left = left.manage(ctx);
                let managed_right = right.manage(ctx);
                match (&managed_left, op, &managed_right) {
                    (Ok((_, Some(x))), _, Ok((_, Some(y)))) => resolved_cond(op.eval(*x, *y)),

                    (Ok((_, Some(true))), BinaryCondOperator::And, _)
                    | (Ok((_, Some(false))), BinaryCondOperator::Or, _) => {
                        let (cond_right, _) = managed_right?;
                        Ok((cond_right, None))
                    }

                    (_, BinaryCondOperator::And, Ok((_, Some(true))))
                    | (_, BinaryCondOperator::Or, Ok((_, Some(false)))) => {
                        let (cond_left, _) = managed_left?;
                        Ok((cond_left, None))
                    }

                    (Ok((_, Some(false))), BinaryCondOperator::And, _)
                    | (_, BinaryCondOperator::And, Ok((_, Some(false)))) => resolved_cond(false),

                    (Ok((_, Some(true))), BinaryCondOperator::Or, _)
                    | (_, BinaryCondOperator::Or, Ok((_, Some(true)))) => resolved_cond(true),

                    _ => {
                        let (left, _) = managed_left?;
                        let (right, _) = managed_right?;
                        let cond = ObsCondition::BinopCondition {
                            left: Box::new(left),
                            op: *op,
                            right: Box::new(right),
                        };
                        Ok((cond, None))
                    }
                }
            }
        }
    }
}

impl Managable for Contract {
    type Output = LifecycleResult;

    fn manage(&self, ctx: &LifecycleContext) -> Result<Self::Output, LifecycleError> {
        match self {
            Self::All(contracts) => {
                let mut managed_contracts = Vec::new();
                let mut flows: Vec<PaidFlow> = Vec::new();
                for contract in contracts {
                    let LifecycleResult {
                        paid_flows,
                        managed_contract,
                    } = contract.manage(ctx)?;
                    paid_flows.into_iter().for_each(|f| flows.push(f));
                    match managed_contract {
                        Contract::All(v) if v.len() == 0 => (),
                        _ => managed_contracts.push(managed_contract),
                    }
                }
                let managed_contract = Contract::All(managed_contracts);
                Ok(LifecycleResult {
                    paid_flows: flows,
                    managed_contract,
                })
            }
            Self::Flow {
                currency,
                date,
                amount,
            } => {
                let (managed_amount, explicit_amount) = amount.manage(ctx)?;
                let default_managed_contract = || Contract::Flow {
                    currency: currency.clone(),
                    date: *date,
                    amount: Box::new(managed_amount),
                };
                match explicit_amount {
                    Some(0.) => Ok(LifecycleResult {
                        paid_flows: Vec::new(),
                        managed_contract: Contract::All(Vec::new()),
                    }),
                    Some(amount) if *date < ctx.as_of => Ok(LifecycleResult {
                        paid_flows: vec![PaidFlow {
                            pay_date: *date,
                            currency: currency.clone(),
                            amount,
                        }],
                        managed_contract: Contract::All(Vec::new()),
                    }),
                    _ => Ok(LifecycleResult {
                        paid_flows: Vec::new(),
                        managed_contract: default_managed_contract(),
                    }),
                }
            }
            Self::IfContract {
                condition,
                true_contract,
                false_contract,
            } => {
                let (cond, cond_resolved) = condition.manage(ctx)?;
                match cond_resolved {
                    Some(true) => true_contract.manage(ctx),
                    Some(false) => false_contract.manage(ctx),
                    None => {
                        let managed_true = true_contract.manage(ctx)?;
                        let managed_false = false_contract.manage(ctx)?;
                        if managed_true.paid_flows.len() > 0 || managed_false.paid_flows.len() > 0 {
                            return Err(LifecycleError::InvalidContract);
                        }
                        let managed_contract = Contract::IfContract {
                            condition: Box::new(cond),
                            true_contract: Box::new(managed_true.managed_contract),
                            false_contract: Box::new(managed_false.managed_contract),
                        };
                        Ok(LifecycleResult {
                            paid_flows: Vec::new(),
                            managed_contract,
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
