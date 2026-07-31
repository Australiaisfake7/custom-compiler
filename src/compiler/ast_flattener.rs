use std::collections::HashMap;

use crate::compiler::parser::{BinaryOp, Expression, LiteralType, Statement, UnaryOp};

enum OpCode {
    PushConst(LiteralType),
    PushScope, PopScope,
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue(usize), JumpIfFalse(usize),
    GetVar(usize), SetVar(usize),
}
enum FlattenError {
    UndeclaredVariable(String),
}

fn flatten_statements(statements: &Vec<Statement>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = Vec::new();
    let mut vars: HashMap<String, usize> = HashMap::new();

    for statement in statements {
        opcodes.append(&mut flatten_statement(statement, &mut vars)?);
    }

    Ok(opcodes)
}

fn flatten_statement(statement: &Statement, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    match statement {
        Statement::Block(statements) => flatten_statements(statements),
        Statement::Expression(expression) => flatten_expression(expression, vars),
    }
}

fn flatten_expression(expression: &Box<Expression>, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    match &**expression {
        Expression::Unary { operator, right } => flatten_unary(operator, right, vars),
        Expression::Binary { left, operator, right } => flatten_binary(left, operator, right, vars),
        Expression::Assignment { target, value } => {
            let mut opcodes: Vec<OpCode> = flatten_expression(value, vars)?;

            match &**target {
                Expression::Variable(i) => {
                    if !vars.contains_key(i) {
                        return Err(FlattenError::UndeclaredVariable(i.clone()));
                    }

                    opcodes.push(OpCode::SetVar(vars.get(i).unwrap().clone()));
                },
                Expression::MemberAccess { class, member } => panic!(),
                _ => unreachable!(),
            }

            Ok(opcodes)
        },
        Expression::Literal(l) => Ok(vec![OpCode::PushConst(l.clone())])
    }
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(right, vars)?;
    
    opcodes.push(match operator {
        UnaryOp::LNot => OpCode::LNot,
        UnaryOp::Negate => OpCode::Negate,
    });

    Ok(opcodes)
}

fn flatten_binary(left: &Box<Expression>, operator: &BinaryOp, right: &Box<Expression>, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars)?;

    opcodes.append(&mut match operator {
        BinaryOp::Add => normal_binary(left, right, OpCode::Add, vars)?,
        BinaryOp::Subtract => normal_binary(left, right, OpCode::Subtract, vars)?,
        BinaryOp::Multiply => normal_binary(left, right, OpCode::Multiply, vars)?,
        BinaryOp::Divide => normal_binary(left, right, OpCode::Divide, vars)?,
        BinaryOp::Equal => normal_binary(left, right, OpCode::Equal, vars)?,
        BinaryOp::Greater => normal_binary(left, right, OpCode::Greater, vars)?,
        BinaryOp::GreaterEqual => normal_binary(left, right, OpCode::GreaterEqual, vars)?,
        BinaryOp::Less => normal_binary(left, right, OpCode::Less, vars)?,
        BinaryOp::LessEqual => normal_binary(left, right, OpCode::LessEqual, vars)?,
        BinaryOp::NotEqual => normal_binary(left, right, OpCode::NotEqual, vars)?,
        BinaryOp::LOr => {
            short_circuit_binary(left, right, true, vars)?
        },
        BinaryOp::LAnd => {
            short_circuit_binary(left, right, false, vars)?
        },
    });

    Ok(opcodes)
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars)?;
    opcodes.append(&mut flatten_expression(right, vars)?);
    opcodes.push(opcode);

    Ok(opcodes)
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, vars: &mut HashMap<String, usize>) -> Result<Vec<OpCode>, FlattenError> {
    let mut opcodes: Vec<OpCode> = flatten_expression(left, vars)?;
    
    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue(0) } else { OpCode::JumpIfFalse(0) });
    opcodes.append(&mut flatten_expression(right, vars)?);
    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue(opcodes.len() - 1) } else { OpCode::JumpIfFalse(opcodes.len() - 1) };

    Ok(opcodes)
}