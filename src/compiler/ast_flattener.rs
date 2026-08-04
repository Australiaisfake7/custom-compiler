use std::collections::HashMap;

use crate::compiler::parser::{BinaryOp, Expression, LiteralType, Statement, UnaryOp};

enum OpCode {
    PushConst(LiteralType), Pop(usize),
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue { index: usize, pop: bool }, JumpIfFalse {index: usize, pop: bool }, Jump(usize),
    GetVar(usize), SetVar(usize),
    Call { index: usize, paremeters: usize }, Return,
    GetMember(String), SetMember(String), CallMember { member: String, parameters: usize },
}
enum FlattenError {
    UndeclaredVariable(String), UndeclaredFunction(String), InvalidFunctionCallee(Box<Expression>),
}

fn flatten_statements(statements: &Vec<Statement>, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    for statement in statements {
        flatten_statement(statement, opcodes, vars, funcs, classes)?;
    }

    Ok(())
}

fn flatten_statement(statement: &Statement, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    match statement {
        Statement::Block(statements) => {
            let start_index: usize = vars.len();
            flatten_statements(statements, opcodes, vars, funcs, classes)?;

            opcodes.push(OpCode::Pop(vars.len() - start_index));
            vars.truncate(start_index);
        }
        Statement::Expression(expression) => { flatten_expression(expression, opcodes, vars, funcs, classes)?; opcodes.push(OpCode::Pop(1)); },
        Statement::If { condition, block } => {
            flatten_expression(condition, opcodes, vars, funcs, classes)?;

            let start_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            flatten_statements(block, opcodes, vars, funcs, classes)?;
            *opcodes.get_mut(start_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
        },
        Statement::IfElse { condition, block1, block2 } => {
            flatten_expression(condition, opcodes, vars, funcs, classes)?;

            let start_index_1: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });
            flatten_statements(block1, opcodes, vars, funcs, classes)?;
            
            let start_index_2: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));
            *opcodes.get_mut(start_index_1).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            flatten_statements(block2, opcodes, vars, funcs, classes)?;
            *opcodes.get_mut(start_index_2).unwrap() = OpCode::Jump(opcodes.len());
        },
    };

    Ok(())
}

fn flatten_expression(expression: &Expression, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    match expression {
        Expression::Unary { operator, right } => flatten_unary(operator, right, opcodes, vars, funcs, classes)?,
        Expression::Binary { left, operator, right } => flatten_binary(left, operator, right, opcodes, vars, funcs, classes)?,
        Expression::Assignment { target, value } => {
            flatten_expression(value, opcodes, vars, funcs, classes)?;

            match &**target {
                Expression::Variable(i) => {
                    opcodes.push(match vars.iter().rposition(|s| s == i) {
                        Some(index) => OpCode::SetVar(index),
                        None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                    });
                },
                Expression::MemberAccess { class, member } => {
                    flatten_expression(class, opcodes, vars, funcs, classes)?;
                    opcodes.push(OpCode::SetMember(member.clone()));
                }
                _ => unreachable!(),
            };
        },
        Expression::Literal(l) => opcodes.push(OpCode::PushConst(l.clone())),
        Expression::Call { callee, parameters } => {
            match &**callee {
                Expression::Variable(i) => {
                    if !funcs.contains_key(i) {
                        return Err(FlattenError::UndeclaredFunction(i.clone()));
                    }

                    for parameter in parameters {
                        flatten_expression(parameter, opcodes, vars, funcs, classes)?;
                    }
                    opcodes.push(OpCode::Call { index: funcs.get(i).unwrap().clone(), paremeters: parameters.len() });
                },
                Expression::MemberAccess { class, member } => {
                    flatten_expression(class, opcodes, vars, funcs, classes)?;

                    for parameter in parameters {
                        flatten_expression(parameter, opcodes, vars, funcs, classes)?;
                    }

                    opcodes.push(OpCode::CallMember { member: member.clone(), parameters: parameters.len() });
                },
                _ => return Err(FlattenError::InvalidFunctionCallee(callee.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            flatten_expression(class, opcodes, vars, funcs, classes)?;

            opcodes.push(OpCode::GetMember(member.clone()));
        },
        Expression::Variable(i) => {
            match vars.iter().rposition(|s| s == i) {
                Some(index) => opcodes.push(OpCode::GetVar(index)),
                None => return Err(FlattenError::UndeclaredVariable(i.clone())),
            }
        },
    };

    Ok(())
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    flatten_expression(right, opcodes, vars, funcs, classes)?;
    
    opcodes.push(match operator {
        UnaryOp::LNot => OpCode::LNot,
        UnaryOp::Negate => OpCode::Negate,
    });

    Ok(())
}

fn flatten_binary(left: &Box<Expression>, operator: &BinaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    match operator {
        BinaryOp::Add => normal_binary(left, right, OpCode::Add, opcodes, vars, funcs, classes)?,
        BinaryOp::Subtract => normal_binary(left, right, OpCode::Subtract, opcodes, vars, funcs, classes)?,
        BinaryOp::Multiply => normal_binary(left, right, OpCode::Multiply, opcodes, vars, funcs, classes)?,
        BinaryOp::Divide => normal_binary(left, right, OpCode::Divide, opcodes, vars, funcs, classes)?,
        BinaryOp::Equal => normal_binary(left, right, OpCode::Equal, opcodes, vars, funcs, classes)?,
        BinaryOp::Greater => normal_binary(left, right, OpCode::Greater, opcodes, vars, funcs, classes)?,
        BinaryOp::GreaterEqual => normal_binary(left, right, OpCode::GreaterEqual, opcodes, vars, funcs, classes)?,
        BinaryOp::Less => normal_binary(left, right, OpCode::Less, opcodes, vars, funcs, classes)?,
        BinaryOp::LessEqual => normal_binary(left, right, OpCode::LessEqual, opcodes, vars, funcs, classes)?,
        BinaryOp::NotEqual => normal_binary(left, right, OpCode::NotEqual, opcodes, vars, funcs, classes)?,
        BinaryOp::LOr => {
            short_circuit_binary(left, right, true, opcodes, vars, funcs, classes)?
        },
        BinaryOp::LAnd => {
            short_circuit_binary(left, right, false, opcodes, vars, funcs, classes)?
        },
    };

    Ok(())
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    flatten_expression(left, opcodes, vars, funcs, classes)?;
    flatten_expression(right, opcodes, vars, funcs, classes)?;
    opcodes.push(opcode);

    Ok(())
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, opcodes: &mut Vec<OpCode>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, usize>) -> Result<(), FlattenError> {
    flatten_expression(left, opcodes, vars, funcs, classes)?;

    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue { index: 0, pop: false } } else { OpCode::JumpIfFalse { index: 0, pop: false } });
    opcodes.push(OpCode::Pop(1));
    flatten_expression(right, opcodes, vars, funcs, classes)?;
    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue { index: opcodes.len(), pop: false } } else { OpCode::JumpIfFalse { index: opcodes.len(), pop: false } };

    Ok(())
}