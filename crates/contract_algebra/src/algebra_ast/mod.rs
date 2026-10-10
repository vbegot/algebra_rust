use common::currency::Currency;
use common::date::Date;

#[derive(Clone, Debug)]
pub enum Contract {
    All(Vec<Contract>),
    Flow {
        currency: Currency,
        date: Date,
        amount: Box<Observable>,
    },
    IfContract {
        condition: Box<ObsCondition>,
        true_contract: Box<Contract>,
        false_contract: Box<Contract>,
    },
}

#[derive(Clone, Debug)]
pub enum Observable {
    Constant(f64),
    Fixing {
        name: String,
        fixing_date: Date,
    },
    BinopObservable {
        left: Box<Observable>,
        op: BinaryOperator,
        right: Box<Observable>,
    },
    UnopObservable {
        op: UnaryOperator,
        obs: Box<Observable>,
    },
    IfObservable {
        condition: Box<ObsCondition>,
        true_observable: Box<Observable>,
        false_observable: Box<Observable>,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Times,
    Div,
    Max,
    Min,
    Pow,
}

impl BinaryOperator {
    pub fn eval(&self, x: f64, y: f64) -> f64 {
        match self {
            Self::Plus => x + y,
            Self::Minus => x - y,
            Self::Times => x * y,
            Self::Div => x / y,
            Self::Max => x.max(y),
            Self::Min => x.min(y),
            Self::Pow => f64::powf(x, y),
        }
    }

    pub fn to_string(&self, x: &str, y: &str) -> String {
        match self {
            Self::Plus => format!("({} + {})", x, y),
            Self::Minus => format!("({} - {})", x, y),
            Self::Times => format!("({} × {})", x, y),
            Self::Div => format!("({} / {})", x, y),
            Self::Max => format!("max({}, {})", x, y),
            Self::Min => format!("min({}, {})", x, y),
            Self::Pow => format!("{}^{}", x, y),
        }
    }
}

impl Observable {
    pub fn lt(self, other: Self) -> ObsCondition {
        ObsCondition::SimpleCondition {
            left: Box::new(self),
            comp: Comparison::Lower,
            right: Box::new(other),
        }
    }

    pub fn leq(self, other: Self) -> ObsCondition {
        ObsCondition::SimpleCondition {
            left: Box::new(self),
            comp: Comparison::LowertOrEqual,
            right: Box::new(other),
        }
    }

    pub fn gt(self, other: Self) -> ObsCondition {
        ObsCondition::SimpleCondition {
            left: Box::new(self),
            comp: Comparison::Higher,
            right: Box::new(other),
        }
    }

    pub fn geq(self, other: Self) -> ObsCondition {
        ObsCondition::SimpleCondition {
            left: Box::new(self),
            comp: Comparison::HigherOrEqual,
            right: Box::new(other),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOperator {
    Neg,
    Log,
    Exp,
    Sqrt,
    Sq,
    Abs,
}

impl UnaryOperator {
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Self::Neg => -x,
            Self::Log => x.ln(),
            Self::Exp => x.exp(),
            Self::Sqrt => x.sqrt(),
            Self::Sq => x * x,
            Self::Abs => x.abs(),
        }
    }

    pub fn to_string(&self, x: &str) -> String {
        match self {
            Self::Neg => format!("-({})-", x),
            Self::Log => format!("log({})", x),
            Self::Exp => format!("exp({})", x),
            Self::Sqrt => format!("√({})", x),
            Self::Sq => format!("({})²", x),
            Self::Abs => format!("|{}|", x),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ObsCondition {
    SimpleCondition {
        left: Box<Observable>,
        comp: Comparison,
        right: Box<Observable>,
    },
    Not(Box<ObsCondition>),
    BinopCondition {
        left: Box<ObsCondition>,
        op: BinaryCondOperator,
        right: Box<ObsCondition>,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum Comparison {
    Lower,
    LowertOrEqual,
    Higher,
    HigherOrEqual,
}

impl Comparison {
    pub fn eval(&self, x: f64, y: f64) -> bool {
        match self {
            Self::Lower => x < y,
            Self::LowertOrEqual => x <= y,
            Self::Higher => x > y,
            Self::HigherOrEqual => x >= y,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Lower => String::from("<"),
            Self::LowertOrEqual => String::from("<="),
            Self::Higher => String::from(">"),
            Self::HigherOrEqual => String::from(">="),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryCondOperator {
    And,
    Or,
}

impl BinaryCondOperator {
    pub fn eval(&self, x: bool, y: bool) -> bool {
        match self {
            Self::And => x && y,
            Self::Or => x || y,
        }
    }
}

#[cfg(test)]
mod tests;

pub mod algebra_dsl;
pub mod algebra_printer;
