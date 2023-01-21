use super::{num::FlexInt, parse::{Node, NodeKind}};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    pub data_type: DataType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DataType {
    pub bits: usize,
    pub signed: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvaluationResult {
    result: FlexInt,
    overflow: bool,
}

impl EvaluationResult {
    pub fn new(result: FlexInt, overflow: bool) -> Self {
        Self { result, overflow }
    }
}

pub fn evaluate(node: &Node, config: &Configuration) -> EvaluationResult {
    match &node.kind {
        NodeKind::Number(num) => EvaluationResult::new(num.clone(), false),
        
        NodeKind::Add(a, b) => {
            let a = evaluate(&a, config);
            let b = evaluate(&b, config);

            let (result, overflow) = a.result.add(&b.result, config.data_type.signed);
            EvaluationResult::new(result, a.overflow || b.overflow || overflow)
        },
        
        NodeKind::Subtract(a, b) => todo!(),
        NodeKind::Divide(a, b) => todo!(),
        NodeKind::Multiply(a, b) => todo!(),
    }
}
