use common::matrix::Matrix;

use crate::compiled_contract::EvalContext;
use crate::compiled_contract::compiler::CompiledContract;
use crate::model_interface::MCModel;

pub struct MCResults {
    pub price: f64,
    pub var: f64,
    pub stdev: f64,
    pub stderr: f64,
}

impl MCResults {
    fn new(sum: f64, sum2: f64, nb_paths: usize) -> Self {
        let nb_paths = nb_paths as f64;
        let price = sum / nb_paths;
        let var = (sum2 - nb_paths * price * price) / (nb_paths - 1.0);
        MCResults {
            price,
            var,
            stdev: var.sqrt(),
            stderr: (var / nb_paths).sqrt(),
        }
    }
}

pub fn price_mc<Model: MCModel>(
    compiled_contract: &CompiledContract,
    model: &mut Model,
    nb_paths: usize,
) -> MCResults {
    assert!(nb_paths > 1, "nb_paths must be strictly higher than 1");
    let compilation_ctx = compiled_contract.get_compilation_context();
    let as_of = compilation_ctx.get_as_of();
    let fix_dates = compilation_ctx.get_fix_dates();
    let underlyings = compilation_ctx.get_underlyings();
    let flow_infos = compilation_ctx.get_flow_infos();
    model.init(&as_of, &fix_dates, &underlyings, flow_infos);
    let nb_fix_dates = fix_dates.len();
    let nb_underlyings = underlyings.len();
    let nb_discounts = flow_infos.len();
    let mut discounts = Vec::new();
    discounts.resize(nb_discounts, 0.0);
    let mut trajectories = Matrix::new(nb_fix_dates, nb_underlyings);
    model.fill_discounts(&mut discounts);
    model.fill_underlyings(trajectories.get_mut_slice(0));
    let mut eval_ctx = EvalContext::new(&mut discounts, &mut trajectories);
    let mut prev_date;
    let mut float_dates = Vec::new();
    float_dates.resize(nb_fix_dates, 0.0);
    fix_dates
        .iter()
        .enumerate()
        .for_each(|(idx, date)| float_dates[idx] = f64::from(*date - as_of) / 365.0);
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    for _ in 0..nb_paths {
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
        sum += traj_price;
        sum2 += traj_price * traj_price;
    }
    return MCResults::new(sum, sum2, nb_paths);
}
