use super::*;

pub fn contract_to_string(contract: &Contract) -> String {
    contract_to_string_aux(contract, "")
}

fn contract_to_string_aux(contract: &Contract, indent: &str) -> String {
    match contract {
        Contract::All(c) if c.len() == 0 => format!("{}nothing", indent),
        Contract::All(contracts) => {
            let mut res = String::new();
            for contract in contracts {
                res.push_str(&contract_to_string_aux(contract, indent));
                res.push_str(";\n");
            }
            res.pop();
            res
        }
        Contract::Flow {
            currency,
            date,
            amount,
        } => format!(
            "{}flow({}, {}, {})",
            indent,
            currency.to_string(),
            date.to_string(),
            observable_to_string(amount)
        ),
        Contract::IfContract {
            condition,
            true_contract,
            false_contract,
        } => {
            let next_indent = format!("    {}", indent);
            format!(
                "{}if {} then:\n{}\n{}else:\n{}\n{}endif",
                indent,
                condition_to_string(condition),
                contract_to_string_aux(true_contract, &next_indent),
                indent,
                contract_to_string_aux(false_contract, &next_indent),
                indent
            )
        }
    }
}

pub fn observable_to_string(observable: &Observable) -> String {
    match observable {
        Observable::Constant(x) => format!("{}", x),
        Observable::Fixing { name, fixing_date } => {
            format!("{}({})", name, fixing_date.to_string())
        }
        Observable::BinopObservable { left, op, right } => {
            op.to_string(&observable_to_string(left), &observable_to_string(right))
        }
        Observable::UnopObservable { op, obs } => op.to_string(&observable_to_string(obs)),
        Observable::IfObservable {
            condition,
            true_observable,
            false_observable,
        } => format!(
            "if {} then {} else {}",
            condition_to_string(condition),
            observable_to_string(true_observable),
            observable_to_string(false_observable)
        ),
    }
}

pub fn condition_to_string(condition: &ObsCondition) -> String {
    match condition {
        ObsCondition::BinopCondition {
            left,
            op: BinaryCondOperator::Or,
            right,
        } => format!(
            "({} | {})",
            condition_to_string(left),
            condition_to_string(right)
        ),
        ObsCondition::BinopCondition {
            left,
            op: BinaryCondOperator::And,
            right,
        } => format!(
            "({} & {})",
            condition_to_string(left),
            condition_to_string(right)
        ),
        ObsCondition::Not(condition) => format!("not({})", condition_to_string(condition)),
        ObsCondition::SimpleCondition { left, comp, right } => format!(
            "{} {} {}",
            observable_to_string(left),
            comp.to_string(),
            observable_to_string(right),
        ),
    }
}
