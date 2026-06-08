use std::task::ready;

use crate::parser::{BinaryOp, Expression, LiteralType, UnaryOp};

pub enum EvaluateError {
    UnexpectedBinaryOperands { operands: (LiteralType, LiteralType), operator: BinaryOp},
    UnexpectedUnaryOperand { operand: LiteralType, operator: UnaryOp},
}

fn evaluate(expression: Expression) -> Result<LiteralType, EvaluateError> {
    return match expression {
        Expression::Literal(t) => Ok(t),
        Expression::Unary { operator: o, right: r} => Ok({
            match *r {
                    Expression::Literal(t) => evaluate_unary(o, t)?,
                    _ => evaluate_unary(o, evaluate(*r)?)?
            }
        }),
        Expression::Binary { left: l, operator: o, right: r } => Ok({
           match (*l, *r) {
                (Expression::Literal(t1), Expression::Literal(t2)) => evaluate_binary(t1, o, t2)?,
                (e1, e2) => evaluate_binary(evaluate(e1)?, o, evaluate(e2)?)?,
            }
        }),
    };
}

fn evaluate_unary(operator: UnaryOp, right: LiteralType) -> Result<LiteralType, EvaluateError> {
    return match (operator, right) {
        (UnaryOp::LNot, LiteralType::Bool(v)) => Ok(LiteralType::Bool(!v)),
        (UnaryOp::Negate, LiteralType::Int(v)) => Ok(LiteralType::Int(-v)),
        (UnaryOp::Negate, LiteralType::Float(v)) => Ok(LiteralType::Float(-v)),
        (o, r) => Err(EvaluateError::UnexpectedUnaryOperand { operand: r, operator: o }),
    };
}

fn evaluate_binary(left: LiteralType, operator: BinaryOp, right: LiteralType) -> Result<LiteralType, EvaluateError> {
    return match (left, operator, right) {
        
    };
}