use crate::algebra_ast::{
    BinaryCondOperator, BinaryOperator, Contract, Currency, Date, ObsCondition, Observable,
};
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};

// Observables

impl Add for Observable {
    type Output = Observable;

    fn add(self, rhs: Observable) -> Self::Output {
        Observable::BinopObservable {
            left: Box::new(self),
            op: BinaryOperator::Plus,
            right: Box::new(rhs),
        }
    }
}

impl Sub for Observable {
    type Output = Observable;

    fn sub(self, rhs: Observable) -> Self::Output {
        Observable::BinopObservable {
            left: Box::new(self),
            op: BinaryOperator::Minus,
            right: Box::new(rhs),
        }
    }
}

impl Mul for Observable {
    type Output = Observable;

    fn mul(self, rhs: Observable) -> Self::Output {
        Observable::BinopObservable {
            left: Box::new(self),
            op: BinaryOperator::Times,
            right: Box::new(rhs),
        }
    }
}

impl Div for Observable {
    type Output = Observable;

    fn div(self, rhs: Observable) -> Self::Output {
        Observable::BinopObservable {
            left: Box::new(self),
            op: BinaryOperator::Div,
            right: Box::new(rhs),
        }
    }
}

pub fn obs(x: f64) -> Observable {
    Observable::Constant(x)
}

pub fn max(x: Observable, y: Observable) -> Observable {
    Observable::BinopObservable {
        left: Box::new(x),
        op: BinaryOperator::Max,
        right: Box::new(y),
    }
}

pub fn min(x: Observable, y: Observable) -> Observable {
    Observable::BinopObservable {
        left: Box::new(x),
        op: BinaryOperator::Min,
        right: Box::new(y),
    }
}

pub fn pow(x: Observable, y: Observable) -> Observable {
    Observable::BinopObservable {
        left: Box::new(x),
        op: BinaryOperator::Pow,
        right: Box::new(y),
    }
}

pub fn if_observable(
    cond: ObsCondition,
    true_obs: Observable,
    false_obs: Observable,
) -> Observable {
    Observable::IfObservable {
        condition: Box::new(cond),
        true_observable: Box::new(true_obs),
        false_observable: Box::new(false_obs),
    }
}

pub fn fixing(name: String, fixing_date: Date) -> Observable {
    Observable::Fixing { name, fixing_date }
}

// Conditions

impl BitAnd for ObsCondition {
    type Output = ObsCondition;

    fn bitand(self, rhs: Self) -> Self::Output {
        ObsCondition::BinopCondition {
            left: Box::new(self),
            op: BinaryCondOperator::And,
            right: Box::new(rhs),
        }
    }
}

impl BitOr for ObsCondition {
    type Output = ObsCondition;

    fn bitor(self, rhs: Self) -> Self::Output {
        ObsCondition::BinopCondition {
            left: Box::new(self),
            op: BinaryCondOperator::Or,
            right: Box::new(rhs),
        }
    }
}

pub fn not(x: ObsCondition) -> ObsCondition {
    ObsCondition::Not(Box::new(x))
}

// Contracts

#[allow(non_upper_case_globals, dead_code)]
const nothing: Contract = Contract::All(Vec::new());

pub fn all(contracts: Vec<Contract>) -> Contract {
    if contracts.len() == 1 {
        contracts[0].clone()
    } else {
        Contract::All(contracts)
    }
}

impl Add for Contract {
    type Output = Contract;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = match self {
            Contract::All(c) => c,
            _ => vec![self],
        };
        res.push(rhs);
        Contract::All(res)
    }
}

pub fn flow(currency: Currency, date: Date, amount: Observable) -> Contract {
    Contract::Flow {
        currency,
        date,
        amount: Box::new(amount),
    }
}

pub fn if_contract(
    cond: ObsCondition,
    true_contract: Contract,
    false_contract: Contract,
) -> Contract {
    Contract::IfContract {
        condition: Box::new(cond),
        true_contract: Box::new(true_contract),
        false_contract: Box::new(false_contract),
    }
}
