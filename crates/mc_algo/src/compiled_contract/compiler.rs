use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display};

use super::*;
use common::{currency::Currency, date::Date};
use contract_algebra::algebra_ast::{Contract, ObsCondition, Observable};

pub struct FlowInfos {
    pub fix_date: Date,
    pub pay_date: Date,
    pub currency: Currency,
}

pub struct CompilationCtx {
    as_of: Date,
    current_date: Date,
    fix_dates: BTreeSet<Date>,
    fix_date_idx: BTreeMap<Date, usize>,
    underlying_idx: BTreeMap<String, usize>,
    flow_infos: Vec<FlowInfos>,
}

impl CompilationCtx {
    pub fn new(contract: &Contract, as_of: &Date) -> Self {
        let fix_dates = get_contract_fix_dates(contract);
        let mut fix_date_idx = BTreeMap::new();
        fix_dates.iter().enumerate().for_each(|(idx, date)| {
            fix_date_idx.insert(*date, idx);
        });
        Self {
            as_of: *as_of,
            current_date: *as_of,
            fix_dates,
            fix_date_idx,
            underlying_idx: BTreeMap::new(),
            flow_infos: Vec::new(),
        }
    }

    fn get_fix_idx(&self, date: &Date) -> Option<usize> {
        let idx = self.fix_date_idx.get(date)?;
        Some(*idx)
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

    fn register_flow(
        &mut self,
        currency: &Currency,
        pay_date: &Date,
        amount: &Observable,
    ) -> usize {
        let amount_fix_date = get_obs_fix_dates(amount).into_iter().max();
        let fix_date = match amount_fix_date {
            Some(date) => self.current_date.max(date),
            None => self.current_date,
        };
        let idx = self.flow_infos.len();
        self.flow_infos.push(FlowInfos {
            fix_date,
            pay_date: *pay_date,
            currency: *currency,
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
                currency.to_string()
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

#[derive(Debug)]
pub enum CompilationError {
    NonMeasurableFlow { fix_date: Date, pay_date: Date },
    BadContext,
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMeasurableFlow { fix_date, pay_date } => write!(
                f,
                "Flow with pay date ({}) before fix date ({}).",
                pay_date.to_string(),
                fix_date.to_string()
            ),
            Self::BadContext => f.write_str("Bad Compilation Context"),
        }
    }
}

impl Error for CompilationError {}

pub trait Compilable {
    type EvalOutput;

    fn compile(
        &self,
        ctx: &mut CompilationCtx,
    ) -> Result<Box<dyn Compiled<Output = Self::EvalOutput>>, CompilationError>;
}

impl Compilable for Contract {
    type EvalOutput = f64;

    fn compile(
        &self,
        ctx: &mut CompilationCtx,
    ) -> Result<Box<dyn Compiled<Output = Self::EvalOutput>>, CompilationError> {
        match self {
            Contract::All(contracts) => {
                let mut compiled_contracts = Vec::new();
                for contract in contracts.iter() {
                    compiled_contracts.push(contract.compile(ctx)?);
                }
                Ok(Box::new(super::CompiledAll {
                    contracts: compiled_contracts,
                }))
            }
            Contract::Flow {
                currency,
                date,
                amount,
            } => {
                let idx = ctx.register_flow(currency, date, amount);
                let fix_date = ctx.flow_infos[idx].fix_date;
                if (fix_date > *date) {
                    return Err(CompilationError::NonMeasurableFlow {
                        fix_date,
                        pay_date: *date,
                    });
                }
                let amount = amount.compile(ctx)?;
                Ok(Box::new(super::CompiledFlow { idx, amount }))
            }
            Contract::IfContract {
                condition,
                true_contract,
                false_contract,
            } => {
                // Explicit clone are note needed as prev_current_date implement Copy
                // but it document that we explicitly want to copy the dates;
                let prev_current_date = ctx.current_date.clone();
                let condition_date = get_cond_fix_dates(condition).into_iter().max();
                let current_date = match condition_date {
                    Some(date) => date.max(ctx.current_date.clone()),
                    None => ctx.current_date.clone(),
                };
                let condition = condition.compile(ctx)?;
                ctx.current_date = current_date;
                let true_contract = true_contract.compile(ctx);
                let false_contract = false_contract.compile(ctx);
                ctx.current_date = prev_current_date;
                Ok(Box::new(CompiledIf {
                    condition,
                    compiled_true: true_contract?,
                    compiled_false: false_contract?,
                }))
            }
        }
    }
}

impl Compilable for Observable {
    type EvalOutput = f64;

    fn compile(
        &self,
        ctx: &mut CompilationCtx,
    ) -> Result<Box<dyn Compiled<Output = Self::EvalOutput>>, CompilationError> {
        match self {
            Observable::Constant(x) => Ok(Box::new(CompiledConstant { value: *x })),
            Observable::Fixing { name, fixing_date } => {
                let ul_idx = ctx.register_underlying(name);
                let date_idx = ctx.get_fix_idx(fixing_date);
                match date_idx {
                    None => Err(CompilationError::BadContext),
                    Some(date_idx) => Ok(Box::new(CompiledFixing { ul_idx, date_idx })),
                }
            }
            Observable::BinopObservable { left, op, right } => panic!("Not Implemented"),
            Observable::UnopObservable { op, obs } => panic!("Not Implemented"),
            Observable::IfObservable {
                condition,
                true_observable,
                false_observable,
            } => panic!("Not Implemented"),
        }
    }
}

impl Compilable for ObsCondition {
    type EvalOutput = bool;

    fn compile(
        &self,
        _ctx: &mut CompilationCtx,
    ) -> Result<Box<dyn Compiled<Output = Self::EvalOutput>>, CompilationError> {
        panic!("Not Implemented")
    }
}
