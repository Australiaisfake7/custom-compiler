use std::collections::HashMap;

use crate::compiler::{ast_flattener::OpCode::SetMember, parser::{BinaryOp, Expression, FunctionData, LiteralType, Statement, UnaryOp}};

enum OpCode {
    PushConst(LiteralType), Pop(usize),
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue { index: usize, pop: bool }, JumpIfFalse {index: usize, pop: bool }, Jump(usize),
    GetVar(usize), SetVar(usize),
    GetGlobal(usize), SetGlobal(usize), DefineGlobal,
    Call { index: usize, paremeters: usize }, Return, Print,
    GetMember(String), SetMember(String), CallMember { member: String, parameters: usize },
    NewStack, PopStack,
    NewInstance(String), Duplicate,
}
enum FlattenError {
    UndeclaredVariable(String), UndeclaredFunction(String), InvalidFunctionCallee(Box<Expression>), ContinueOutsideLoop,
    Shadowing(String), FunctionDeclarationInsideScope(String), ClassDeclarationInsideScope(String), UnexpectedClassMember(Statement),
}

struct CompiledClassData {
    vars: Vec<String>,
    funcs: HashMap<String, usize>,
    parent: Option<String>,
    constructor: usize,
}

fn flatten_statements(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>, loop_starts: &mut Vec<(usize, usize)>, depth: usize) -> Result<(), FlattenError> {
    for statement in statements {
        flatten_statement(statement, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
    }

    Ok(())
}

fn flatten_statement(statement: &Statement, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>, loop_starts: &mut Vec<(usize, usize)>, depth: usize) -> Result<(), FlattenError> {
    match statement {
        Statement::Block(statements) => {
            flatten_block(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
        }
        Statement::Expression(expression) => { flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?; opcodes.push(OpCode::Pop(1)); },
        Statement::If { condition, block } => {
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

            let start_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
            *opcodes.get_mut(start_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
        },
        Statement::IfElse { condition, block1, block2 } => {
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

            let start_index_1: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });
            flatten_block(block1, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;

            let start_index_2: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));
            *opcodes.get_mut(start_index_1).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            flatten_block(block2, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
            *opcodes.get_mut(start_index_2).unwrap() = OpCode::Jump(opcodes.len());
        },
        Statement::Return(expression) => {
            match expression {
                Some(e) => flatten_expression(e, opcodes, global_vars, vars, funcs, classes)?,
                None => flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, vars, funcs, classes)?,
            };

            opcodes.push(OpCode::PopStack);
            opcodes.push(OpCode::Return);
        },
        Statement::Continue => {
            let (start_index, vars_len) = match loop_starts.last() {
                Some(loop_start) => (loop_start.0, loop_start.1),
                None => return Err(FlattenError::ContinueOutsideLoop),
            };

            opcodes.push(OpCode::Pop(vars.len() - vars_len));
            opcodes.push(OpCode::Jump(start_index));
        },
        Statement::While { condition, block } => {
            let start_index: usize = opcodes.len();
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((start_index, vars.len()));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
            loop_starts.pop();

            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
        },
        Statement::For { initializer, condition, update, block } => {
            let outer_vars_len: usize = vars.len();
            flatten_statement(initializer, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1)?;
            let inner_vars_len: usize = vars.len();

            let skip_index: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));

            flatten_statement(update, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1)?;
            
            *opcodes.get_mut(skip_index).unwrap() = OpCode::Jump(opcodes.len());

            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;
            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((skip_index + 1, inner_vars_len));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth)?;
            loop_starts.pop();

            opcodes.push(OpCode::Jump(skip_index + 1));

            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
            opcodes.push(OpCode::Pop(vars.len() - outer_vars_len));
            vars.truncate(outer_vars_len);
        },
        Statement::Print(expression) => {
            flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?;
            opcodes.push(OpCode::Print);
        },
        Statement::Declaration { name, value, data_type } => {
            if depth == 0 && global_vars.iter().any(|s| s == name) || depth != 0 && vars.iter().any(|s| s == name) {
                return Err(FlattenError::Shadowing(name.clone()));
            }

            flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;

            if depth != 0 {
                vars.push(name.clone());
            }
            else {
                opcodes.push(OpCode::DefineGlobal);
                global_vars.push(name.clone());
            }
        },
        Statement::Function { name, data } => {
            if depth != 0 {
                return Err(FlattenError::FunctionDeclarationInsideScope(name.to_owned()))
            }
            if funcs.contains_key(name) {
                return Err(FlattenError::Shadowing(name.to_owned())); 
            }

            funcs.insert(name.clone(), flatten_function(data, opcodes, global_vars, funcs, classes, loop_starts, depth)?);
        },
        Statement::Class { name, block, parent } => {
            if depth != 0 {
                return Err(FlattenError::ClassDeclarationInsideScope(name.clone()));
            }
            if classes.contains_key(name) {
                return Err(FlattenError::Shadowing(name.clone()));
            }

            let class: CompiledClassData = CompiledClassData { vars: Vec::new(), funcs: HashMap::new(), parent: parent.clone(), constructor: 0 };
            classes.insert(name.clone(), class);

            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));
            opcodes.push(OpCode::NewStack);
            opcodes.push(OpCode::NewInstance(name.clone()));

            for member in block {
                match member {
                    Statement::Declaration { name: var_name, value, data_type: _ } => {
                        if classes.get(name).unwrap().vars.iter().any(|s| s == var_name) {
                            return Err(FlattenError::Shadowing(var_name.clone()));
                        }

                        classes.get_mut(name).unwrap().vars.push(var_name.clone());
                        opcodes.push(OpCode::Duplicate);
                        flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;
                        opcodes.push(SetMember(var_name.clone()));
                        opcodes.push(OpCode::Pop(1));
                    },
                    Statement::Function { name: func_name, data } => {
                        if classes.get(name).unwrap().funcs.contains_key(func_name) {
                            return Err(FlattenError::Shadowing(func_name.clone()));
                        }

                        classes.get_mut(name).unwrap().funcs.insert(func_name.clone(), flatten_function(data, opcodes, global_vars, funcs, classes, loop_starts, depth)?);
                    },
                    statement => return Err(FlattenError::UnexpectedClassMember(statement.clone())),
                }
            }

            opcodes.push(OpCode::PopStack);
            opcodes.push(OpCode::Return);
            *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());
            classes.get_mut(name).unwrap().constructor = jump_index + 1;
        }
    };

    Ok(())
}

fn flatten_block(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>, loop_starts: &mut Vec<(usize, usize)>, depth: usize) -> Result<(), FlattenError> {
    let start_index: usize = vars.len();
    flatten_statements(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1)?;

    opcodes.push(OpCode::Pop(vars.len() - start_index));
    vars.truncate(start_index);

    Ok(())
}

fn flatten_expression(expression: &Expression, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>) -> Result<(), FlattenError> {
    match expression {
        Expression::Unary { operator, right } => flatten_unary(operator, right, opcodes, global_vars, vars, funcs, classes)?,
        Expression::Binary { left, operator, right } => flatten_binary(left, operator, right, opcodes, global_vars, vars, funcs, classes)?,
        Expression::Assignment { target, value } => {
            match &**target {
                Expression::Variable(i) => {
                    flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;
                    opcodes.push(match vars.iter().rposition(|s| s == i) {
                        Some(index) => OpCode::SetVar(index),
                        None => match global_vars.iter().rposition(|s| s == i) {
                            Some(index) => OpCode::SetGlobal(index),
                            None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                        }
                    });
                },
                Expression::MemberAccess { class, member } => {
                    flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;
                    flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;
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
                        flatten_expression(parameter, opcodes, global_vars, vars, funcs, classes)?;
                    }
                    opcodes.push(OpCode::Call { index: funcs.get(i).unwrap().clone(), paremeters: parameters.len() });
                },
                Expression::MemberAccess { class, member } => {
                    flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

                    for parameter in parameters {
                        flatten_expression(parameter, opcodes, global_vars, vars, funcs, classes)?;
                    }

                    opcodes.push(OpCode::CallMember { member: member.clone(), parameters: parameters.len() });
                },
                _ => return Err(FlattenError::InvalidFunctionCallee(callee.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

            opcodes.push(OpCode::GetMember(member.clone()));
        },
        Expression::Variable(i) => {
            match vars.iter().rposition(|s| s == i) {
                Some(index) => opcodes.push(OpCode::GetVar(index)),
                None => match global_vars.iter().rposition(|s| s == i) {
                    Some(index) => opcodes.push(OpCode::GetGlobal(index)),
                    None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                }
            }
        },
    };

    Ok(())
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>) -> Result<(), FlattenError> {
    flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;
    
    opcodes.push(match operator {
        UnaryOp::LNot => OpCode::LNot,
        UnaryOp::Negate => OpCode::Negate,
    });

    Ok(())
}

fn flatten_binary(left: &Box<Expression>, operator: &BinaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>) -> Result<(), FlattenError> {
    match operator {
        BinaryOp::Add => normal_binary(left, right, OpCode::Add, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Subtract => normal_binary(left, right, OpCode::Subtract, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Multiply => normal_binary(left, right, OpCode::Multiply, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Divide => normal_binary(left, right, OpCode::Divide, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Equal => normal_binary(left, right, OpCode::Equal, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Greater => normal_binary(left, right, OpCode::Greater, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::GreaterEqual => normal_binary(left, right, OpCode::GreaterEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Less => normal_binary(left, right, OpCode::Less, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::LessEqual => normal_binary(left, right, OpCode::LessEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::NotEqual => normal_binary(left, right, OpCode::NotEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::LOr => {
            short_circuit_binary(left, right, true, opcodes, global_vars, vars, funcs, classes)?
        },
        BinaryOp::LAnd => {
            short_circuit_binary(left, right, false, opcodes, global_vars, vars, funcs, classes)?
        },
    };

    Ok(())
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>) -> Result<(), FlattenError> {
    flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;
    flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;
    opcodes.push(opcode);

    Ok(())
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>) -> Result<(), FlattenError> {
    flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;

    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue { index: 0, pop: false } } else { OpCode::JumpIfFalse { index: 0, pop: false } });
    opcodes.push(OpCode::Pop(1));
    flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;
    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue { index: opcodes.len(), pop: false } } else { OpCode::JumpIfFalse { index: opcodes.len(), pop: false } };

    Ok(())
}

fn flatten_function(data: &FunctionData, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<String>, funcs: &mut HashMap<String, usize>, classes: &mut HashMap<String, CompiledClassData>, loop_starts: &mut Vec<(usize, usize)>, depth: usize) -> Result<usize, FlattenError> {
    let jump_index: usize = opcodes.len();
    opcodes.push(OpCode::Jump(0));

    let mut func_vars: Vec<String> = Vec::new();

    for (_d, s) in &data.parameters {
        if func_vars.iter().any(|var| var == s) {
            return Err(FlattenError::Shadowing(s.clone()));
        }
        func_vars.push(s.clone());
    }

    opcodes.push(OpCode::NewStack);

    flatten_block(&data.block, opcodes, global_vars, &mut func_vars, funcs, classes, loop_starts, depth)?;
    flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, &mut func_vars, funcs, classes)?;
    opcodes.push(OpCode::PopStack);
    opcodes.push(OpCode::Return);

    *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());

    Ok(jump_index + 1)
}