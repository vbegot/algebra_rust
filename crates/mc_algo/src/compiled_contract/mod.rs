use common::matrix::Matrix;
use contract_algebra::algebra_ast::{
    BinaryCondOperator, BinaryOperator, Comparison, UnaryOperator,
};

type CompiledFloat = Box<dyn Compiled<Output = f64>>;
type CompiledBool = Box<dyn Compiled<Output = bool>>;

pub struct EvalContext<'a> {
    discounts: &'a mut [f64], // discount[i] represents N(0)/N(t) P(t, T) FX(t) for the i-th flow
    trajectories: &'a Matrix,
}

impl<'a> EvalContext<'a> {
    pub fn new(discounts: &'a mut [f64], trajectories: &'a Matrix) -> Self {
        Self {
            discounts,
            trajectories,
        }
    }
}

pub trait Compiled {
    type Output;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output;
}

// Compiled Contracts

struct CompiledAll {
    contracts: Vec<CompiledFloat>,
}

struct CompiledFlow {
    idx: usize,
    amount: CompiledFloat,
}

// This struct can be shared between contracts and obs
struct CompiledIf {
    condition: CompiledBool,
    compiled_true: CompiledFloat,
    compiled_false: CompiledFloat,
}

impl Compiled for CompiledAll {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        let mut res = 0.0;
        self.contracts.iter().for_each(|c| res += c.eval(ctx));
        res
    }
}

impl Compiled for CompiledFlow {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        self.amount.eval(ctx) * ctx.discounts[self.idx]
    }
}

impl Compiled for CompiledIf {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        if self.condition.eval(ctx) {
            self.compiled_true.eval(ctx)
        } else {
            self.compiled_false.eval(ctx)
        }
    }
}

// Compiled Observables

struct CompiledConstant {
    value: f64,
}

struct CompiledFixing {
    ul_idx: usize,
    date_idx: usize,
}

struct CompiledBinopObservable {
    left: CompiledFloat,
    op: BinaryOperator,
    right: CompiledFloat,
}

struct CompiledUnopObservable {
    op: UnaryOperator,
    obs: CompiledFloat,
}

impl Compiled for CompiledConstant {
    type Output = f64;

    fn eval(&self, _ctx: &mut EvalContext) -> Self::Output {
        self.value
    }
}

impl Compiled for CompiledFixing {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        ctx.trajectories[(self.date_idx, self.ul_idx)]
    }
}

impl Compiled for CompiledBinopObservable {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        let left = self.left.eval(ctx);
        let right = self.right.eval(ctx);
        self.op.eval(left, right)
    }
}

impl Compiled for CompiledUnopObservable {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        self.op.eval(self.obs.eval(ctx))
    }
}

// Compiled Conditions

struct CompiledSimpleCondition {
    left: CompiledFloat,
    comp: Comparison,
    right: CompiledFloat,
}

// Temporary. Can be removed, the "not" can be statically resolved at compilation
struct CompiledNot {
    condition: CompiledBool,
}

struct CompiledBinopCondition {
    left: CompiledBool,
    op: BinaryCondOperator,
    right: CompiledBool,
}

impl Compiled for CompiledSimpleCondition {
    type Output = bool;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        let left = self.left.eval(ctx);
        let right = self.right.eval(ctx);
        self.comp.eval(left, right)
    }
}

impl Compiled for CompiledNot {
    type Output = bool;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        !self.condition.eval(ctx)
    }
}

impl Compiled for CompiledBinopCondition {
    type Output = bool;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        match self.op {
            BinaryCondOperator::And => self.left.eval(ctx) && self.right.eval(ctx),
            BinaryCondOperator::Or => self.left.eval(ctx) || self.right.eval(ctx),
        }
    }
}

pub mod compiler;
