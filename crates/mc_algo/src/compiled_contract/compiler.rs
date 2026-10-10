use std::collections::{BTreeMap, BTreeSet};

use super::*;
use common::date::Date;
use contract_algebra::algebra_ast::{Contract, ObsCondition, Observable};

pub struct FlowInfos {
    pub fix_date: Date,
    pub pay_date: Date,
    pub currency: String,
}

pub struct CompilationCtx {
    as_of: Date,
    current_date: Date,
    fix_dates: BTreeSet<Date>,
    underlying_idx: BTreeMap<String, usize>,
    flow_infos: Vec<FlowInfos>,
}

impl CompilationCtx {
    pub fn new(contract: &Contract, as_of: &Date) -> Self {
        let fix_dates = get_contract_fix_dates(contract);
        Self {
            as_of: *as_of,
            current_date: *as_of,
            fix_dates,
            underlying_idx: BTreeMap::new(),
            flow_infos: Vec::new(),
        }
    }

    fn register_underlying(&mut self, underlying: &str) -> usize {
        let idx = self.underlying_idx.get(underlying);
        match idx {
            Some(idx) => *idx,
            None => {
                let idx = self.underlying_idx.len();
                self.underlying_idx.insert(underlying.to_string(), idx);
                idx
            }
        }
    }

    fn register_flow(&mut self, currency: &str, pay_date: &Date, amount: &Observable) -> usize {
        let amount_fix_date = get_obs_fix_dates(amount).into_iter().max();
        let fix_date = match amount_fix_date {
            Some(date) => self.current_date.max(date),
            None => self.current_date,
        };
        let idx = self.flow_infos.len();
        self.flow_infos.push(FlowInfos {
            fix_date,
            pay_date: *pay_date,
            currency: currency.to_string(),
        });
        idx
    }

    pub fn show(&self) {
        println!("--------------------\nCompilation Context\n--------------------");
        println!("  As Of: {}", self.as_of.to_string());
        println!("\n  Fix Dates:");
        for (idx, date) in self.fix_dates.iter().enumerate() {
            println!("    {} => {};", idx, date.to_string());
        }
        println!("\n  Underlyings:");
        for (underlying, idx) in self.underlying_idx.iter() {
            println!("    {} => {};", idx, underlying);
        }
        println!("\n  Flow Infos:");
        for (
            idx,
            FlowInfos {
                fix_date,
                pay_date,
                currency,
            },
        ) in self.flow_infos.iter().enumerate()
        {
            println!(
                "    {} => Fix: {}, Pay: {}, Currency: {}",
                idx,
                fix_date.to_string(),
                pay_date.to_string(),
                currency
            )
        }
        println!("--------------------\n")
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
