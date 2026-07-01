use std::{collections::HashMap, mem};

use crate::compiler::parser::FunctionData;

use super::{lexer::DataType, parser::{BinaryOp, Expression, LiteralType, Statement, UnaryOp}};

#[derive(Debug)]
#[allow(dead_code)]
pub enum EvaluateError {
    UnexpectedBinaryOperands { operands: (LiteralType, LiteralType), operator: BinaryOp},
    UnexpectedUnaryOperand { operand: LiteralType, operator: UnaryOp},
    IdentifierShadowing(String),
    UndeclaredVariable(String),
    UndeclaredFunction(String),
    UnexpectedCondition(LiteralType),
    UnexpectedVariableValueType { expected: DataType, recieved: LiteralType },
    UnexpectedFunctionCallee(Expression),
    UnexpectedParameterCount { callee: Expression, expected: usize, got: usize },
    UnexpectedParameterType { callee: Expression, expected: DataType, got: LiteralType },
}

pub fn evaluate_statements(statements: &Vec<Statement>, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData>) -> Result<(), EvaluateError> {
    for statement in statements.iter() {
        evaluate_statement(statement, global_vars, vars, funcs)?;
    }

    Ok(())
}

fn evaluate_statement(statement: &Statement, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData>) -> Result<(), EvaluateError> {
    return match statement {
        Statement::Declaration { name: n, value: v , data_type: t} => {
            if is_declared(n, vars) {
                Err(EvaluateError::IdentifierShadowing(n.to_owned()))
            }
            else {
                let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs)?;

                match (&value, t) {
                    (LiteralType::Int(_), DataType::Int) => (),
                    (LiteralType::Float(_), DataType::Float) => (),
                    (LiteralType::String(_), DataType::String) => (),
                    (LiteralType::Bool(_), DataType::Bool) => (),
                    (LiteralType::Null, _) => (),
                    (other_val, other_type) => return Err(EvaluateError::UnexpectedVariableValueType { expected: other_type.clone(), recieved: other_val.clone() })
                }

                if let Some(h) = vars.last_mut() {
                    h.insert(n.to_owned(), value);
                }
                Ok(())
            }
        },
        Statement::Expression(expr) => {
            let _ = evaluate_expression(expr, global_vars, vars, funcs)?;
            Ok(())
        },
        Statement::Block(stmts) => {
            vars.push(HashMap::new());
            let result: Result<(), EvaluateError> = evaluate_statements(stmts, global_vars, vars, funcs);
            vars.pop();
            result
        },
        Statement::If { condition: c, block} => {
            match evaluate_expression(c, global_vars, vars, funcs)? {
                LiteralType::Bool(b) => {
                    if b {
                        evaluate_statements(block, global_vars, vars, funcs)?;
                    }
                    return Ok(());
                },
                other => return Err(EvaluateError::UnexpectedCondition(other)),
            }
        },
        Statement::IfElse { condition: c, block1, block2} => {
            match evaluate_expression(c, global_vars, vars, funcs)? {
                LiteralType::Bool(b) => {
                    if b {
                        evaluate_statements(block1, global_vars, vars, funcs)?;
                    }
                    else {
                        evaluate_statements(block2, global_vars, vars, funcs)?;
                    }
                    return Ok(());
                },
                other => return Err(EvaluateError::UnexpectedCondition(other)),
            }
        },
        Statement::Print(s) => { println!("{:?}", evaluate_expression(s, global_vars, vars, funcs)?); Ok(()) },
        Statement::While { condition: c, block } => {
            loop {
                match evaluate_expression(c, global_vars, vars, funcs)? {
                    LiteralType::Bool(b) => {
                        if b {
                            evaluate_statements(block, global_vars, vars, funcs)?;
                        }
                        else {
                            break;
                        }
                    },
                    other => return Err(EvaluateError::UnexpectedCondition(other)),
                };
            }

            Ok(())
        },
        Statement::For {initializer: i, condition: c, update: u, block} => {
            evaluate_statement(i, global_vars, vars, funcs)?;

            loop {
                let bool_condition: bool = match evaluate_expression(c, global_vars, vars, funcs)? {
                    LiteralType::Bool(b) => b,
                    other => return Err(EvaluateError::UnexpectedCondition(other)),
                };

                if bool_condition {
                    evaluate_statements(block, global_vars, vars, funcs)?;
                    evaluate_statement(u, global_vars, vars, funcs)?;
                }
                else {
                    break;
                }
            }

            Ok(())
        },
        Statement::Function { name, data } => {
            if funcs.contains_key(name) {
                return Err(EvaluateError::IdentifierShadowing(name.clone()));
            }
            funcs.insert(name.to_owned(), data.clone());

            Ok(())
        }
    }; 
}

fn evaluate_expression(expression: &Expression, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData> ) -> Result<LiteralType, EvaluateError> {
    match expression {
        Expression::Literal(t) => Ok(t.clone()),
        Expression::Unary { operator: o, right: r} => evaluate_unary(o, &evaluate_expression(r, global_vars, vars, funcs)?),
        
        Expression::Binary { left: l, operator: o, right: r } => evaluate_binary(&evaluate_expression(l, global_vars, vars, funcs)?, o, &evaluate_expression(r, global_vars, vars, funcs)?),
        Expression::Assignment { name: n, value: v } => {
            if is_declared(n, vars) {
                let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs)?;
                assign_var(n, &value, vars)?;
                Ok(value)
            }
            else {
                Err(EvaluateError::UndeclaredVariable(n.to_owned()))
            }
        },
        Expression::Variable(n) => {
            match get_var(n, vars) {
                Some(v) => Ok(v),
                None => Err(EvaluateError::UndeclaredVariable(n.to_owned()))
            } 
        },
        Expression::Call { callee, parameters } => {
            match *callee.clone() {
                Expression::Variable(n) => {
                    if !funcs.contains_key(&n) {
                        return Err(EvaluateError::UndeclaredFunction(n));
                    }

                    let mut func_vars: Vec<HashMap<String, LiteralType>> = vec![HashMap::new()];
                    let func_data: FunctionData = funcs.get(&n).unwrap().clone();

                    if parameters.len() != func_data.parameters.len() {
                        return Err(EvaluateError::UnexpectedParameterCount { callee: *callee.clone(), expected: func_data.parameters.len(), got: parameters.len() });
                    }
                    for (i, (d, s)) in func_data.parameters.iter().enumerate() {
                        let p: LiteralType = match parameters.get(i) {
                                Some(v) => evaluate_expression(v, global_vars, vars, funcs)?,
                                None => unreachable!(),
                        };
                        if DataType::try_from(&p).map(|data_type| &data_type != d).unwrap_or(false) {
                            return Err(EvaluateError::UnexpectedParameterType { callee: *callee.clone(), expected: d.clone(), got: p });
                        }

                        func_vars.first_mut().unwrap().insert(s.clone(), p);
                    }

                    evaluate_statements(&func_data.block, global_vars, &mut func_vars, funcs)?;

                    Ok(LiteralType::Null)
                },
               other => Err(EvaluateError::UnexpectedFunctionCallee(other)),
            }
        }
    }
}

fn is_declared(name: &str, vars: &Vec<HashMap<String, LiteralType>>) -> bool {
    for h in vars.iter() {
        if let Some(_) = h.get(name) {
            return true;
        }
    }

    return false;
}

fn get_var(name: &str, vars: &Vec<HashMap<String, LiteralType>>) -> Option<LiteralType> {
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

fn evaluate_unary(operator: &UnaryOp, right: &LiteralType) -> Result<LiteralType, EvaluateError> {
    return match (operator.clone(), right.clone()) {
        (UnaryOp::LNot, LiteralType::Bool(v)) => Ok(LiteralType::Bool(!v)),
        (UnaryOp::Negate, LiteralType::Int(v)) => Ok(LiteralType::Int(-v)),
        (UnaryOp::Negate, LiteralType::Float(v)) => Ok(LiteralType::Float(-v)),
        (o, r) => Err(EvaluateError::UnexpectedUnaryOperand { operand: r, operator: o }),
    };
}

fn evaluate_binary(left: &LiteralType, operator: &BinaryOp, right: &LiteralType) -> Result<LiteralType, EvaluateError> {
    return match (left.clone(), operator.clone(), right.clone()) {
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

        (LiteralType::Bool(v1), BinaryOp::LAnd, LiteralType::Bool(v2)) => Ok(LiteralType::Bool(v1 && v2)),
        (LiteralType::Bool(v1), BinaryOp::LOr, LiteralType::Bool(v2)) => Ok(LiteralType::Bool(v1 || v2)),

        (l, o, r) => Err(EvaluateError::UnexpectedBinaryOperands{ operands: (l, r), operator: o }),
    };
}