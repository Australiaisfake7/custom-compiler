use std::collections::HashMap;

use crate::parser::{BinaryOp, Expression, Statement, LiteralType, UnaryOp};

#[derive(Debug)]
#[allow(dead_code)]
pub enum EvaluateError {
    UnexpectedBinaryOperands { operands: (LiteralType, LiteralType), operator: BinaryOp},
    UnexpectedUnaryOperand { operand: LiteralType, operator: UnaryOp},
    VariableShadowing(String),
    UndeclaredVariable(String),
}

pub fn evaluate_statment(statement: Statement, vars: &mut [HashMap<String, LiteralType>]) -> Result<(), EvaluateError> {
    return match statement {
        Statement::Declaration { name: n, value: v , data_type: t} => {
            if is_declared(&n, vars) {
                Err(EvaluateError::VariableShadowing(n))
            }
            else {
                let value: LiteralType = evaluate_expression(*v, vars)?;
                if let Some(h) = vars.last_mut() {
                    h.insert(n, value);
                }
                Ok(())
            }
        },
        Statement::Expression(_) => Ok(()),
    } 
}

fn evaluate_expression(expression: Expression, vars: &mut [HashMap<String, LiteralType>]) -> Result<LiteralType, EvaluateError> {
    return match expression {
        Expression::Literal(t) => Ok(t),
        Expression::Unary { operator: o, right: r} => Ok({
            match *r {
                    Expression::Literal(t) => evaluate_unary(o, t)?,
                    _ => evaluate_unary(o, evaluate_expression(*r, vars)?)?
            }
        }),
        Expression::Binary { left: l, operator: o, right: r } => Ok({
           match (*l, *r) {
                (Expression::Literal(t1), Expression::Literal(t2)) => evaluate_binary(t1, o, t2)?,
                (e1, e2) => evaluate_binary(evaluate_expression(e1, vars)?, o, evaluate_expression(e2, vars)?)?,
            }
        }),
        Expression::Assignment { name: n, value: v } => {
            if is_declared(&n, vars) {
                let value: LiteralType = evaluate_expression(*v, vars)?;
                assign_var(&n, &value, vars)?;
                Ok(value)
            }
            else {
                Err(EvaluateError::UndeclaredVariable(n))
            }
        },
        Expression::Variable(n) => {
            match get_var(&n, vars) {
                Some(v) => Ok(v),
                None => Err(EvaluateError::UndeclaredVariable(n))
            } 
        } 
    };
}

fn is_declared(name: &str, vars: &[HashMap<String, LiteralType>]) -> bool {
    for h in vars.iter() {
        if let Some(_) = h.get(name) {
            return true;
        }
    }

    return false;
}

fn get_var(name: &str, vars: &[HashMap<String, LiteralType>]) -> Option<LiteralType> {
    for h in vars.iter() {
        if let Some(v) = h.get(name) {
            return Some(v.clone());
        }
    }

    return None;
}

    fn assign_var(name: &str, value: &LiteralType, vars: &mut [HashMap<String, LiteralType>]) -> Result<(), EvaluateError> {
        for h in vars.iter_mut() {
        if let Some(_) = h.get(name) {
            h.insert(name.to_owned(), value.clone());
            return Ok(());
        }
    }

    return Err(EvaluateError::UndeclaredVariable(name.to_owned()));
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
        (LiteralType::Int(v1), BinaryOp::Add, LiteralType::Int(v2)) => Ok(LiteralType::Int(v1 + v2)),
        (LiteralType::Float(v1), BinaryOp::Add, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 + v2)),
        (LiteralType::Int(v1), BinaryOp::Add, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 as f64 + v2)),
        (LiteralType::Float(v1), BinaryOp::Add, LiteralType::Int(v2)) => Ok(LiteralType::Float(v1 + v2 as f64)),
        (LiteralType::String(v1), BinaryOp::Add, LiteralType::String(v2)) => Ok(LiteralType::String(format!("{}{}", v1, v2))),

        (LiteralType::Int(v1), BinaryOp::Subtract, LiteralType::Int(v2)) => Ok(LiteralType::Int(v1 - v2)),
        (LiteralType::Float(v1), BinaryOp::Subtract, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 - v2)),
        (LiteralType::Int(v1), BinaryOp::Subtract, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 as f64 - v2)),
        (LiteralType::Float(v1), BinaryOp::Subtract, LiteralType::Int(v2)) => Ok(LiteralType::Float(v1 - v2 as f64)),

        (LiteralType::Int(v1), BinaryOp::Multiply, LiteralType::Int(v2)) => Ok(LiteralType::Int(v1 * v2)),
        (LiteralType::Float(v1), BinaryOp::Multiply, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 * v2)),
        (LiteralType::Int(v1), BinaryOp::Multiply, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 as f64 * v2)),
        (LiteralType::Float(v1), BinaryOp::Multiply, LiteralType::Int(v2)) => Ok(LiteralType::Float(v1 * v2 as f64)),

        (LiteralType::Int(v1), BinaryOp::Divide, LiteralType::Int(v2)) => Ok(LiteralType::Int(v1 / v2)),
        (LiteralType::Float(v1), BinaryOp::Divide, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 / v2)),
        (LiteralType::Int(v1), BinaryOp::Divide, LiteralType::Float(v2)) => Ok(LiteralType::Float(v1 as f64 / v2)),
        (LiteralType::Float(v1), BinaryOp::Divide, LiteralType::Int(v2)) => Ok(LiteralType::Float(v1 / v2 as f64)),

        (LiteralType::Int(v1), BinaryOp::Equal, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 == v2)),
        (LiteralType::Float(v1), BinaryOp::Equal, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 == v2)),
        (LiteralType::Int(v1), BinaryOp::Equal, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 == v2)),
        (LiteralType::Float(v1), BinaryOp::Equal, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 == v2 as f64)),

        (LiteralType::Int(v1), BinaryOp::NotEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 != v2)),
        (LiteralType::Float(v1), BinaryOp::NotEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 != v2)),
        (LiteralType::Int(v1), BinaryOp::NotEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 != v2)),
        (LiteralType::Float(v1), BinaryOp::NotEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 != v2 as f64)),

        (LiteralType::Int(v1), BinaryOp::Less, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 < v2)),
        (LiteralType::Float(v1), BinaryOp::Less, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 < v2)),
        (LiteralType::Int(v1), BinaryOp::Less, LiteralType::Float(v2)) => Ok(LiteralType::Bool((v1 as f64) < v2)),
        (LiteralType::Float(v1), BinaryOp::Less, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 < v2 as f64)),
        
        (LiteralType::Int(v1), BinaryOp::LessEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 <= v2)),
        (LiteralType::Float(v1), BinaryOp::LessEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 <= v2)),
        (LiteralType::Int(v1), BinaryOp::LessEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 <= v2)),
        (LiteralType::Float(v1), BinaryOp::LessEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 <= v2 as f64)),
        
        (LiteralType::Int(v1), BinaryOp::Greater, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 > v2)),
        (LiteralType::Float(v1), BinaryOp::Greater, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 > v2)),
        (LiteralType::Int(v1), BinaryOp::Greater, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 > v2)),
        (LiteralType::Float(v1), BinaryOp::Greater, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 > v2 as f64)),
        
        (LiteralType::Int(v1), BinaryOp::GreaterEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 >= v2)),
        (LiteralType::Float(v1), BinaryOp::GreaterEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 >= v2)),
        (LiteralType::Int(v1), BinaryOp::GreaterEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 >= v2)),
        (LiteralType::Float(v1), BinaryOp::GreaterEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 >= v2 as f64)),
        
        (l, o, r) => Err(EvaluateError::UnexpectedBinaryOperands{ operands: (l, r), operator: o }),
    };
}