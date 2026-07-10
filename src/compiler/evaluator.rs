use std::{cell::RefCell, collections::HashMap, mem, rc::Rc};
use crate::compiler::parser::{ClassData, FunctionData};

use super::{lexer::DataType, parser::{BinaryOp, Expression, LiteralType, Statement, UnaryOp}};

#[derive(Debug)]
pub enum EvaluateError {
    UnexpectedBinaryOperands { operands: (LiteralType, LiteralType), operator: BinaryOp},
    UnexpectedUnaryOperand { operand: LiteralType, operator: UnaryOp},
    IdentifierShadowing(String),
    UndeclaredVariable(String),
    UndeclaredVariableInClass(String),
    UndeclaredFunction(String),
    UndeclaredClass(String),
    UnexpectedCondition(LiteralType),
    UnexpectedVariableValueType { expected: DataType, recieved: LiteralType },
    UnexpectedFunctionCallee(Expression),
    UnexpectedParameterCount { callee: Expression, expected: usize, got: usize },
    UnexpectedParameterType { callee: Expression, expected: DataType, got: LiteralType },
    UnexpectedStatementInClass { class: String, statement: Statement },
    ExpressionIsNotClass(Box<Expression>),
}
pub enum ControlFlow {
    None,
    Return(Option<LiteralType>),
}
pub fn evaluate_statements(statements: &Vec<Statement>, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, ClassData>) -> Result<ControlFlow, EvaluateError> {
    for statement in statements.iter() {
        if let ControlFlow::Return(expr) = evaluate_statement(statement, global_vars, vars, funcs, classes)? {
            return Ok(ControlFlow::Return(expr));
        }
    }

    Ok(ControlFlow::None)
}

fn evaluate_statement(statement: &Statement, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, ClassData>) -> Result<ControlFlow, EvaluateError> {
    return match statement {
        Statement::Declaration { name: n, value: v , data_type: t} => {
            if vars.iter().any(|map| map.contains_key(n)) {
                Err(EvaluateError::IdentifierShadowing(n.to_owned()))
            }
            else {
                let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;

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
                Ok(ControlFlow::None)
            }
        },
        Statement::Expression(expr) => {
            let _ = evaluate_expression(expr, global_vars, vars, funcs, classes)?;
            Ok(ControlFlow::None)
        },
        Statement::Block(stmts) => {
            vars.push(HashMap::new());
            let result: Result<ControlFlow, EvaluateError> = evaluate_statements(stmts, global_vars, vars, funcs, classes);
            vars.pop();
            result.map(|c| ControlFlow::None)
        },
        Statement::If { condition: c, block} => {
            match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                LiteralType::Bool(b) => {
                    if b {
                        evaluate_statements(block, global_vars, vars, funcs, classes)?;
                    }
                    return Ok(ControlFlow::None);
                },
                other => return Err(EvaluateError::UnexpectedCondition(other)),
            }
        },
        Statement::IfElse { condition: c, block1, block2} => {
            match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                LiteralType::Bool(b) => {
                    if b {
                        evaluate_statements(block1, global_vars, vars, funcs, classes)?;
                    }
                    else {
                        evaluate_statements(block2, global_vars, vars, funcs, classes)?;
                    }
                    return Ok(ControlFlow::None);
                },
                other => return Err(EvaluateError::UnexpectedCondition(other)),
            }
        },
        Statement::Print(s) => { println!("{:?}", evaluate_expression(s, global_vars, vars, funcs, classes)?); Ok(ControlFlow::None) },
        Statement::While { condition: c, block } => {
            loop {
                match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                    LiteralType::Bool(b) => {
                        if b {
                            evaluate_statements(block, global_vars, vars, funcs, classes)?;
                        }
                        else {
                            break;
                        }
                    },
                    other => return Err(EvaluateError::UnexpectedCondition(other)),
                };
            }

            Ok(ControlFlow::None)
        },
        Statement::For {initializer: i, condition: c, update: u, block} => {
            evaluate_statement(i, global_vars, vars, funcs, classes)?;

            loop {
                let bool_condition: bool = match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                    LiteralType::Bool(b) => b,
                    other => return Err(EvaluateError::UnexpectedCondition(other)),
                };

                if bool_condition {
                    evaluate_statements(block, global_vars, vars, funcs, classes)?;
                    evaluate_statement(u, global_vars, vars, funcs, classes)?;
                }
                else {
                    break;
                }
            }

            Ok(ControlFlow::None)
        },
        Statement::Function { name, data } => {
            if funcs.contains_key(name) {
                return Err(EvaluateError::IdentifierShadowing(name.clone()));
            }
            funcs.insert(name.to_owned(), data.clone());

            Ok(ControlFlow::None)
        },
        Statement::Return(expr) => {
            Ok(ControlFlow::Return(expr.as_ref().map(|e| evaluate_expression(e, global_vars, vars, funcs, classes)).transpose()?))
        },
        Statement::Class { name, block } => {
            let mut class_vars: HashMap<String, LiteralType> = HashMap::new();
            let mut class_funcs: HashMap<String, FunctionData> = HashMap::new();
            
            for statement in block {
                match statement {
                    Statement::Declaration { name: n, value, data_type } => {
                        if class_vars.contains_key(n) {
                            return Err(EvaluateError::IdentifierShadowing(n.clone()));
                        }

                        let v: LiteralType = evaluate_expression(value, global_vars, vars, funcs, classes)?;

                        if DataType::try_from(&v).map(|t| mem::discriminant(&t) != mem::discriminant(data_type)).unwrap_or(false) {
                            return Err(EvaluateError::UnexpectedVariableValueType { expected: data_type.clone(), recieved: v })
                        }

                        class_vars.insert(n.clone(), v);
                    },
                    Statement::Function { name: n, data } => {
                        if class_funcs.contains_key(n) {
                            return Err(EvaluateError::IdentifierShadowing(n.clone()))
                        }

                        class_funcs.insert(n.clone(), data.clone());
                    },
                    other => return Err(EvaluateError::UnexpectedStatementInClass { class: name.clone(), statement: other.clone() }),
                }
            }

            classes.insert(name.clone(), ClassData { vars: class_vars, funcs: class_funcs });

            Ok(ControlFlow::None)
        }
    };
}

fn evaluate_expression(expression: &Expression, global_vars: &mut HashMap<String, LiteralType>, vars: &mut Vec<HashMap<String, LiteralType>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, ClassData>) -> Result<LiteralType, EvaluateError> {
    match expression {
        Expression::Literal(t) => Ok(t.clone()),
        Expression::Unary { operator: o, right: r} => evaluate_unary(o, &evaluate_expression(r, global_vars, vars, funcs, classes)?),
        
        Expression::Binary { left: l, operator: o, right: r } => evaluate_binary(&evaluate_expression(l, global_vars, vars, funcs, classes)?, o, &evaluate_expression(r, global_vars, vars, funcs, classes)?),
        Expression::Assignment { target: t, value: v } => {
            match &**t {
                Expression::Variable(name) => { 
                    let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;
                    if let Some(hash_map) = vars.iter_mut().find(|map| map.contains_key(name)) {
                        hash_map.insert(name.clone() ,value.clone());
                        Ok(value)
                    }
                    else {
                        Err(EvaluateError::UndeclaredVariable(name.to_owned()))
                    }
                },
                Expression::MemeberAccess { class, member } => {
                    let class_data: Rc<RefCell<ClassData>> = match evaluate_expression(class, global_vars, vars, funcs, classes)? {
                        LiteralType::Class(c) => c,
                        _ => return Err(EvaluateError::ExpressionIsNotClass(class.clone()))
                    };
                    if !class_data.borrow().vars.contains_key(member) {
                        return Err(EvaluateError::UndeclaredVariableInClass(member.clone()))
                    }


                    let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;
                    class_data.borrow_mut().vars.insert(member.clone(), value);

                    Ok(class_data.borrow().vars.get(member).unwrap().clone())

                },
                _ => unreachable!() 
            }
        },
        Expression::Variable(n) => {
            match vars.iter().rev().find(|map| map.contains_key(n)).map(|map| map.get(n).unwrap()) {
                Some(v) => Ok(v.clone()),
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
                            Some(v) => evaluate_expression(v, global_vars, vars, funcs, classes)?,
                            None => unreachable!(),
                        };
                        if DataType::try_from(&p).map(|data_type| &data_type != d).unwrap_or(false) {
                            return Err(EvaluateError::UnexpectedParameterType { callee: *callee.clone(), expected: d.clone(), got: p });
                        }

                        func_vars.first_mut().unwrap().insert(s.clone(), p);
                    }

                    match evaluate_statements(&func_data.block, global_vars, &mut func_vars, funcs, classes)? {
                        ControlFlow::None => Ok(LiteralType::Null),
                        ControlFlow::Return(expr) => Ok(expr.unwrap_or(LiteralType::Null))
                    }
                },
               other => Err(EvaluateError::UnexpectedFunctionCallee(other)),
            }
        },
        Expression::MemeberAccess { class, member } => {
            let class_data: Rc<RefCell<ClassData>> = match evaluate_expression(class, global_vars, vars, funcs, classes)? {
                LiteralType::Class(c) => c,
                _ => return Err(EvaluateError::ExpressionIsNotClass(class.clone()))
            };

            if !class_data.borrow().vars.contains_key(member) {
                return Err(EvaluateError::UndeclaredVariableInClass(member.clone()))
            }

            Ok(class_data.borrow().vars.get(member).unwrap().clone())
        }
    }
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