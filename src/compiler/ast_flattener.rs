use std::collections::HashMap;

use crate::compiler::parser::{BinaryOp, Expression, LiteralType, Statement, UnaryOp};

enum OpCode {
    PushConst(LiteralType), Pop(usize),
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue { index: usize, pop: bool }, JumpIfFalse {index: usize, pop: bool }, Jump(usize),
    GetVar(usize), SetVar(usize),
    Call { index: usize, paremeters: usize }, Return,
    GetMember(String), SetMember(String),
}
enum FlattenError {
    UndeclaredVariable(String), UndeclaredFunction(String), InvalidFunctionCallee(Box<Expression>),
}

fn flatten_statements(statements: &Vec<Statement>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = Vec::new();

    for statement in statements {
        opcodes.append(&mut flatten_statement(statement, vars, funcs, classes)?);
    }

    Ok(opcodes)
}

fn flatten_statement(statement: &Statement, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    match statement {
        Statement::Block(statements) => {
            let start_index: usize = vars.len();
            let mut opcodes: Vec<OpCode> = flatten_statements(statements, vars, funcs, classes)?;

            opcodes.push(OpCode::Pop(vars.len() - start_index));
            vars.truncate(start_index);

            Ok(opcodes)
        }
        Statement::Expression(expression) => { let mut opcodes: Vec<OpCode> = flatten_expression(expression, vars, funcs, classes)?; opcodes.push(OpCode::Pop(1)); Ok(opcodes) },
        Statement::If { condition, block } => {
            let mut opcodes: Vec<OpCode> = flatten_expression(condition, vars, funcs, classes)?;

            let start_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            opcodes.append(&mut flatten_statements(block, vars, funcs, classes)?);
            *opcodes.get_mut(start_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            Ok(opcodes)
        },
        Statement::IfElse { condition, block1, block2 } => {
            let mut opcodes: Vec<OpCode> = flatten_expression(condition, vars, funcs, classes)?;

            let start_index_1: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });
            opcodes.append(&mut flatten_statements(block1, vars, funcs, classes)?);
            
            let start_index_2: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));
            *opcodes.get_mut(start_index_1).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            opcodes.append(&mut flatten_statements(block2, vars, funcs, classes)?);
            *opcodes.get_mut(start_index_2).unwrap() = OpCode::Jump(opcodes.len());

            Ok(opcodes)
        },
    }
}

fn flatten_expression(expression: &Expression, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    match expression {
        Expression::Unary { operator, right } => flatten_unary(operator, right, vars, funcs, classes),
        Expression::Binary { left, operator, right } => flatten_binary(left, operator, right, vars, funcs, classes),
        Expression::Assignment { target, value } => {
            let mut opcodes: Vec<OpCode> = flatten_expression(value, vars, funcs, classes)?;

            match &**target {
                Expression::Variable(i) => {
                    opcodes.push(match vars.iter().rposition(|s| s == i) {
                        Some(index) => OpCode::SetVar(index),
                        None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                    })
                },
                Expression::MemberAccess { class, member } => {
                    opcodes.append(&mut flatten_expression(class, vars, funcs, classes)?);
                    opcodes.push(OpCode::SetMember(member.clone()));
                }
                _ => unreachable!(),
            }

            Ok(opcodes)
        },
        Expression::Literal(l) => Ok(vec![OpCode::PushConst(l.clone())]),
        Expression::Call { callee, parameters } => {
            match &**callee {
                Expression::Variable(i) => {
                    if !funcs.contains_key(i) {
                        return Err(FlattenError::UndeclaredFunction(i.clone()));
                    }

                    let mut opcodes: Vec<OpCode> = Vec::new();

                    for parameter in parameters {
                        opcodes.append(&mut flatten_expression(parameter, vars, funcs, classes)?);
                    }
                    opcodes.push(OpCode::Call { index: funcs.get(i).unwrap().clone(), paremeters: parameters.len() });

                    Ok(opcodes)
                },
                Expression::MemberAccess { class, member } => {
                    let mut opcodes: Vec<OpCode> = Vec::new();
                    opcodes.append(&mut flatten_expression(class, vars, funcs, classes)?);

                    for parameter in parameters {
                        opcodes.append(&mut flatten_expression(parameter, vars, funcs, classes)?);
                    }

                    opcodes.push(OpCode::GetMember(member.clone()));
                    Ok(opcodes)
                },
                _ => Err(FlattenError::InvalidFunctionCallee(callee.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            let mut opcodes: Vec<OpCode> = flatten_expression(class, vars, funcs, classes)?;

            opcodes.push(OpCode::GetMember(member.clone()));
            Ok(opcodes)
        },
        Expression::Variable(i) => {
            match vars.iter().rposition(|s| s == i) {
                Some(index) => Ok(vec![OpCode::GetVar(index)]),
                None => Err(FlattenError::UndeclaredVariable(i.clone())),
            }
        },
    }
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(right, vars, funcs, classes)?;
    
    opcodes.push(match operator {
        UnaryOp::LNot => OpCode::LNot,
        UnaryOp::Negate => OpCode::Negate,
    });

    Ok(opcodes)
}

fn flatten_binary(left: &Box<Expression>, operator: &BinaryOp, right: &Box<Expression>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars, funcs, classes)?;

    opcodes.append(&mut match operator {
        BinaryOp::Add => normal_binary(left, right, OpCode::Add, vars, funcs, classes)?,
        BinaryOp::Subtract => normal_binary(left, right, OpCode::Subtract, vars, funcs, classes)?,
        BinaryOp::Multiply => normal_binary(left, right, OpCode::Multiply, vars, funcs, classes)?,
        BinaryOp::Divide => normal_binary(left, right, OpCode::Divide, vars, funcs, classes)?,
        BinaryOp::Equal => normal_binary(left, right, OpCode::Equal, vars, funcs, classes)?,
        BinaryOp::Greater => normal_binary(left, right, OpCode::Greater, vars, funcs, classes)?,
        BinaryOp::GreaterEqual => normal_binary(left, right, OpCode::GreaterEqual, vars, funcs, classes)?,
        BinaryOp::Less => normal_binary(left, right, OpCode::Less, vars, funcs, classes)?,
        BinaryOp::LessEqual => normal_binary(left, right, OpCode::LessEqual, vars, funcs, classes)?,
        BinaryOp::NotEqual => normal_binary(left, right, OpCode::NotEqual, vars, funcs, classes)?,
        BinaryOp::LOr => {
            short_circuit_binary(left, right, true, vars, funcs, classes)?
        },
        BinaryOp::LAnd => {
            short_circuit_binary(left, right, false, vars, funcs, classes)?
        },
    });

    Ok(opcodes)
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars, funcs, classes)?;
    opcodes.append(&mut flatten_expression(right, vars, funcs, classes)?);
    opcodes.push(opcode);

    Ok(opcodes)
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars, funcs, classes)?;

    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue { index: 0, pop: false } } else { OpCode::JumpIfFalse { index: 0, pop: false } });
    opcodes.push(OpCode::Pop);
    opcodes.append(&mut flatten_expression(right, vars, funcs, classes)?);
    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue { index: opcodes.len(), pop: false } } else { OpCode::JumpIfFalse { index: opcodes.len(), pop: false } };

    Ok(opcodes)
}