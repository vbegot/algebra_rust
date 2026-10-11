use crate::compiled_contract::EvalContext;
use crate::compiled_contract::compiler::CompiledContract;
use crate::model_interface::MCModel;
use crate::pricing_algo::MCError::CantInitializeModel;
use common::matrix::Matrix;
use std::error::Error;
use std::time::Instant;

pub struct MCResults {
    pub price: f64,
    pub variance: f64,
    pub stdev: f64,
    pub stderr: f64,
    pub time: f64,
}

impl MCResults {
    fn new(price: f64, m2: f64, nb_paths: usize, timer: &std::time::Instant) -> Self {
        let variance = m2 / (nb_paths - 1) as f64;
        MCResults {
            price,
            variance,
            stdev: variance.sqrt(),
            stderr: (variance / nb_paths as f64).sqrt(),
            time: timer.elapsed().as_secs_f64() * 1_000.0,
        }
    }
}

#[derive(Debug)]
pub enum MCError<ModelError: Error> {
    NotEnoughTrajectories,
    CantInitializeModel(ModelError),
}

impl<ModelError: Error> std::fmt::Display for MCError<ModelError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotEnoughTrajectories => f.write_str("Number of trajectories must be > 1"),
            Self::CantInitializeModel(err) => write!(f, "Can't initialize model: {err}"),
        }
    }
}

impl<ModelError: Error> Error for MCError<ModelError> {}

pub fn price_mc<Model: MCModel>(
    compiled_contract: &CompiledContract,
    model: &mut Model,
    nb_paths: usize,
) -> Result<MCResults, MCError<Model::ModelError>> {
    let timer = Instant::now();
    if nb_paths <= 1 {
        return Err(MCError::NotEnoughTrajectories);
    }
    let compilation_ctx = compiled_contract.get_compilation_context();
    let as_of = compilation_ctx.get_as_of();
    let fix_dates = compilation_ctx.get_fix_dates();
    let underlyings = compilation_ctx.get_underlyings();
    let flow_infos = compilation_ctx.get_flow_infos();
    model
        .init(&as_of, &fix_dates, &underlyings, flow_infos)
        .map_err(CantInitializeModel)?;
    let nb_fix_dates = fix_dates.len();
    let nb_underlyings = underlyings.len();
    let nb_discounts = flow_infos.len();
    let mut discounts = vec![0.0; nb_discounts];
    let mut trajectories = Matrix::new(nb_fix_dates, nb_underlyings);
    model.fill_discounts(&mut discounts);
    model.fill_underlyings(trajectories.get_mut_slice(0));
    let mut eval_ctx = EvalContext::new(&mut discounts, &mut trajectories);
    let mut prev_date;
    let float_dates: Vec<f64> = fix_dates
        .iter()
        .map(|date| f64::from(*date - as_of) / 365.0)
        .collect();
    let mut mean = 0.0;
    let mut m2 = 0.0;
    for path_idx in 0..nb_paths {
        model.init_trajectory();
        prev_date = 0.0;
        for idx in 1..nb_fix_dates {
            let date = float_dates[idx];
            model.simulate(idx, prev_date, date);
            let discounts = eval_ctx.get_discounts();
            model.fill_discounts(discounts);
            let trajectory_slice = eval_ctx.get_trajectory_slice(idx);
            model.fill_underlyings(trajectory_slice);
            prev_date = date;
        }
        let traj_price = compiled_contract.eval(&mut eval_ctx);
        let n = (path_idx + 1) as f64;
        let delta = traj_price - mean;
        mean += delta / n;
        m2 += delta * (traj_price - mean);
    }
    Ok(MCResults::new(mean, m2, nb_paths, &timer))
}
