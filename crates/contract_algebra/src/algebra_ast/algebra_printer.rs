use super::*;

fn contract_to_string(contract: &Contract) -> String {
    contract_to_string_aux(contract, "")
}

fn contract_to_string_aux(contract: &Contract, indent: &str) -> String {
    match contract {
        Contract::All(c) if c.is_empty() => format!("{}nothing", indent),
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
            currency,
            date,
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
        Contract::Option {
            exercise_date,
            given,
            exercise,
            no_exercise,
        } => {
            let next_indent = format!("    {}", indent);
            let prefix = if *given { "counterparty " } else { "" };
            format!(
                "{indent}{prefix}choose at {exercise_date} between:\n{}\nand:\n{}endchoose",
                contract_to_string_aux(exercise, &next_indent),
                contract_to_string_aux(no_exercise, &next_indent)
            )
        }
    }
}

fn observable_to_string(observable: &Observable) -> String {
    match observable {
        Observable::Constant(x) => format!("{}", x),
        Observable::Fixing { name, fixing_date } => {
            format!("{}({})", name, fixing_date)
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

fn condition_to_string(condition: &ObsCondition) -> String {
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
            comp,
            observable_to_string(right),
        ),
    }
}

impl std::fmt::Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&contract_to_string(self))
    }
}

impl std::fmt::Display for Observable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&observable_to_string(self))
    }
}

impl std::fmt::Display for ObsCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&condition_to_string(self))
    }
}
