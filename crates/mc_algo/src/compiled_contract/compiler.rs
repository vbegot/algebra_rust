use std::collections::{BTreeMap, BTreeSet};

use super::*;
use common::date::Date;
use contract_algebra::algebra_ast::{Contract, ObsCondition, Observable};

pub struct CompilationCtx {
    fix_dates: BTreeSet<Date>,
    underlying_idx: BTreeMap<String, usize>,
}

impl CompilationCtx {
    pub fn new(contract: &Contract) -> Self {
        let fix_dates = get_contract_fix_dates(contract);
        Self {
            fix_dates,
            underlying_idx: BTreeMap::new(),
        }
    }
}

fn get_contract_fix_dates(contract: &Contract) -> BTreeSet<Date> {
    match contract {
        Contract::All(contracts) => {
            let mut res = BTreeSet::new();
            let insert = |c| {
                res.extend(get_contract_fix_dates(c));
            };
            contracts.iter().for_each(insert);
            res
        }
        Contract::Flow { amount, .. } => get_obs_fix_dates(amount),
        Contract::IfContract {
            condition,
            true_contract,
            false_contract,
        } => {
            let mut res = get_cond_fix_dates(condition);
            res.extend(get_contract_fix_dates(true_contract));
            res.extend(get_contract_fix_dates(false_contract));
            res
        }
    }
}

fn get_obs_fix_dates(observable: &Observable) -> BTreeSet<Date> {
    match observable {
        Observable::Constant(_) => BTreeSet::new(),
        Observable::Fixing { fixing_date, .. } => BTreeSet::from([*fixing_date]),
        Observable::BinopObservable { left, right, .. } => {
            let mut res = get_obs_fix_dates(left);
            res.extend(get_obs_fix_dates(right));
            res
        }
        Observable::UnopObservable { obs, .. } => get_obs_fix_dates(obs),
        Observable::IfObservable {
            condition,
            true_observable,
            false_observable,
        } => {
            let mut res = get_cond_fix_dates(condition);
            res.extend(get_obs_fix_dates(true_observable));
            res.extend(get_obs_fix_dates(false_observable));
            res
        }
    }
}

fn get_cond_fix_dates(condition: &ObsCondition) -> BTreeSet<Date> {
    match condition {
        ObsCondition::SimpleCondition { left, right, .. } => {
            let mut res = get_obs_fix_dates(left);
            res.extend(get_obs_fix_dates(right));
            res
        }
        ObsCondition::Not(c) => get_cond_fix_dates(c),
        ObsCondition::BinopCondition { left, right, .. } => {
            let mut res = get_cond_fix_dates(left);
            res.extend(get_cond_fix_dates(right));
            res
        }
    }
}

pub trait Compilable {
    type EvalOutput;

    fn compile(&self, ctx: &mut CompilationCtx) -> Box<dyn Compiled<Output = Self::EvalOutput>>;
}

impl Compilable for Contract {
    type EvalOutput = f64;

    fn compile(&self, _ctx: &mut CompilationCtx) -> Box<dyn Compiled<Output = Self::EvalOutput>> {
        Box::new(CompiledAll {
            contracts: Vec::new(),
        })
    }
}
