use super::*;
use contract_algebra::algebra_ast::Contract;

pub struct CompilationCtx {}

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
