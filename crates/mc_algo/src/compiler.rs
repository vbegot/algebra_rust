struct EvalContext {}

struct CompiledAll {
    contracts: Vec<Box<dyn Compiled<Output = f64>>>,
}

struct CompiledFlow {}

struct CompiledIfContract {}

trait Compiled {
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

    fn eval(&self, _ctx: &mut EvalContext) -> Self::Output {
        0.0
    }
}

impl Compiled for CompiledIfContract {
    type Output = f64;

    fn eval(&self, _ctx: &mut EvalContext) -> Self::Output {
        0.0
    }
}
