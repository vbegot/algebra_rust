use common::matrix::Matrix;

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

struct CompiledAll {
    contracts: Vec<CompiledFloat>,
}

struct CompiledFlow {
    idx: usize,
    amount: CompiledFloat,
}

struct CompiledIfContract {
    condition: CompiledBool,
    true_contract: CompiledFloat,
    false_contract: CompiledFloat,
}

pub trait Compiled {
    type Output;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output;
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

impl Compiled for CompiledIfContract {
    type Output = f64;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        if self.condition.eval(ctx) {
            self.true_contract.eval(ctx)
        } else {
            self.false_contract.eval(ctx)
        }
    }
}

pub mod compiler;
