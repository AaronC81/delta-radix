use alloc::{format, string::String};

use super::parse::{Node, NodeKind};
use flex_int::FlexInt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    pub data_type: DataType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DataType {
    pub bits: usize,
    pub signed: bool,
}

impl DataType {
    pub fn concise_name(&self) -> String {
        format!("{}{}", if self.signed { "S" } else { "U" }, self.bits)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvaluationResult {
    pub result: FlexInt,
    pub overflow: bool,
}

impl EvaluationResult {
    pub fn new(result: FlexInt, overflow: bool) -> Self {
        Self { result, overflow }
    }
}

pub fn evaluate(node: &Node, config: &Configuration) -> EvaluationResult {
    match &node.kind {
        NodeKind::Number(num) => EvaluationResult::new(num.clone(), false),
        
        NodeKind::Add(a, b)
        | NodeKind::Subtract(a, b)
        | NodeKind::Divide(a, b)
        | NodeKind::Multiply(a, b) => {
            let a = evaluate(a, config);
            let b = evaluate(b, config);

            let (result, overflow) = match &node.kind {
                NodeKind::Add(_, _) => a.result.add(&b.result, config.data_type.signed),
                NodeKind::Subtract(_, _) => a.result.subtract(&b.result, config.data_type.signed),
                NodeKind::Multiply(_, _) => a.result.multiply(&b.result, config.data_type.signed),
                NodeKind::Divide(_, _) => a.result.divide(&b.result, config.data_type.signed),
                _ => unreachable!()
            };

            EvaluationResult::new(result, a.overflow || b.overflow || overflow)
        },
    }
}
