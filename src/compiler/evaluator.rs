use std::{collections::HashMap, mem, rc::Rc, cell::RefCell};
use crate::compiler::{lexer::DataType, parser::{BinaryOp, ClassData, Expression, FunctionData, InstanceData, LiteralType, Statement, UnaryOp, VariableData}};

#[derive(Debug)]
pub enum EvaluateError {
    UnexpectedBinaryOperands { operands: (LiteralType, LiteralType), operator: BinaryOp},
    UnexpectedUnaryOperand { operand: LiteralType, operator: UnaryOp},
    IdentifierShadowing(String),
    UndeclaredVariable(String),
    UndeclaredVariableInClass { class: Box<Expression>, variable: String },
    UndeclaredFunction(String),
    UndeclaredFunctionInClass{ class: Box<Expression>, function: String },
    UndeclaredClass(String),
    UnexpectedCondition(LiteralType),
    UnexpectedVariableValueType { expected: DataType, recieved: LiteralType },
    UnexpectedReturnValueType { expected: Option<DataType>, recieved: LiteralType },
    UnexpectedFunctionCallee(Expression),
    UnexpectedParameterCount { callee: Box<Expression>, expected: usize, got: usize },
    UnexpectedParameterType { callee: Box<Expression>, expected: DataType, got: LiteralType },
    UnexpectedStatementInClass { class: String, statement: Statement },
    ExpressionIsNotClass { expr: Box<Expression>, value: LiteralType },
    DivisionByZero { dividend: Box<Expression>, divisor: Box<Expression> },
    DeclarationInsideScope(Statement),
    ContinueStatementNotInLoop,
}
pub enum ControlFlow {
    None,
    Return(Option<LiteralType>),
    Continue,
}
pub fn evaluate_statements(statements: &[Statement], global_vars: &mut HashMap<String, VariableData>, vars: &mut Vec<HashMap<String, VariableData>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, Rc<ClassData>>) -> Result<ControlFlow, EvaluateError> {
    for statement in statements.iter() {
        match evaluate_statement(statement, global_vars, vars, funcs, classes)? {
            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
            ControlFlow::Continue => return Ok(ControlFlow::Continue),
            ControlFlow::None => (),
        }
    }

    Ok(ControlFlow::None)
}

fn evaluate_statement(statement: &Statement, global_vars: &mut HashMap<String, VariableData>, vars: &mut Vec<HashMap<String, VariableData>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, Rc<ClassData>>) -> Result<ControlFlow, EvaluateError> {
    match statement {
        Statement::Declaration { name: n, value: v , data_type: t} => {
            if vars.iter().any(|map| map.contains_key(n)) || global_vars.contains_key(n) {
                return Err(EvaluateError::IdentifierShadowing(n.to_owned()));
            }
            let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;

            if !is_value_type_valid(&value, t) {
                return Err(EvaluateError::UnexpectedVariableValueType { expected: t.clone(), recieved: value.clone() });
            }

            if vars.len() == 0 {
                global_vars.insert(n.clone(), VariableData { data_type: t.clone(), value });
            }
            else {
                vars.last_mut().unwrap().insert(n.clone(), VariableData { data_type: t.clone(), value });
            }    

            Ok(ControlFlow::None)
        },
        Statement::Expression(expr) => {
            let _ = evaluate_expression(expr, global_vars, vars, funcs, classes)?;
            Ok(ControlFlow::None)
        },
        Statement::Block(statements) => {
            evaluate_block(statements, global_vars, vars, funcs, classes)
        },
        Statement::If { condition: c, block} => {
            match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                LiteralType::Bool(b) => {
                    if b {
                        match evaluate_block(block, global_vars, vars, funcs, classes)? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Continue => return Ok(ControlFlow::Continue),
                            ControlFlow::None => (),
                        }
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
                        match evaluate_block(block1, global_vars, vars, funcs, classes)? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Continue => return Ok(ControlFlow::Continue),
                            ControlFlow::None => (),
                        }
                    }
                    else {
                        match evaluate_block(block2, global_vars, vars, funcs, classes)? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Continue => return Ok(ControlFlow::Continue),
                            ControlFlow::None => (),
                        }
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
                            match evaluate_block(block, global_vars, vars, funcs, classes)? {
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => (),
                            }
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
            vars.push(HashMap::new());
            evaluate_statement(i, global_vars, vars, funcs, classes)?;

            loop {
                let bool_condition: bool = match evaluate_expression(c, global_vars, vars, funcs, classes)? {
                    LiteralType::Bool(b) => b,
                    other => return Err(EvaluateError::UnexpectedCondition(other)),
                };

                if bool_condition {
                    match evaluate_block(block, global_vars, vars, funcs, classes)? {
                        ControlFlow::Return(v) => { vars.pop(); return Ok(ControlFlow::Return(v)); },
                        _ => (),
                    }
                    evaluate_statement(u, global_vars, vars, funcs, classes)?;
                }
                else {
                    break;
                }
            }

            vars.pop();

            Ok(ControlFlow::None)
        },
        Statement::Function { name, data } => {
            if vars.len() != 0 {
                return Err(EvaluateError::DeclarationInsideScope(statement.clone()));
            }
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
            if classes.contains_key(name) {
                return Err(EvaluateError::IdentifierShadowing(name.clone()))
            }
            if vars.len() != 0 {
                return Err(EvaluateError::DeclarationInsideScope(statement.clone()));
            }

            let mut class_vars: HashMap<String, (DataType, Box<Expression>)> = HashMap::new();
            let mut class_funcs: HashMap<String, FunctionData> = HashMap::new();

            for statement in block {
                match statement {
                    Statement::Declaration { name: n, value, data_type } => {
                        if class_vars.contains_key(n) {
                            return Err(EvaluateError::IdentifierShadowing(name.clone() + "." + n));
                        }

                        class_vars.insert(n.clone(), (data_type.clone(), value.clone()));
                    },
                    Statement::Function { name: n, data } => {
                        if n == "new" || class_funcs.contains_key(n) {
                            return Err(EvaluateError::IdentifierShadowing(name.clone() + "." + n));
                        }

                        class_funcs.insert(n.clone(), data.clone());
                    },
                    other => return Err(EvaluateError::UnexpectedStatementInClass { class: name.clone(), statement: other.clone() }),
                }
            }

            classes.insert(name.clone(), Rc::new(ClassData { vars: class_vars, funcs: class_funcs }));

            Ok(ControlFlow::None)
        },
        Statement::Continue => Ok(ControlFlow::Continue),
    }
}

fn evaluate_expression(expression: &Expression, global_vars: &mut HashMap<String, VariableData>, vars: &mut Vec<HashMap<String, VariableData>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, Rc<ClassData>>) -> Result<LiteralType, EvaluateError> {
    match expression {
        Expression::Literal(t) => Ok(t.clone()),
        Expression::Unary { operator: o, right: r} => evaluate_unary(o, &evaluate_expression(r, global_vars, vars, funcs, classes)?),
        
        Expression::Binary { left: l, operator: o, right: r } => {
            let left_value: LiteralType = evaluate_expression(l, global_vars, vars, funcs, classes)?;

            if left_value == LiteralType::Bool(true) && *o == BinaryOp::LOr || left_value == LiteralType::Bool(false) && *o == BinaryOp::LAnd {
                return Ok(left_value);
            }

            let right_value: LiteralType = evaluate_expression(r, global_vars, vars, funcs, classes)?;
            
            if o == &BinaryOp::Divide && (right_value == LiteralType::Int(0) || right_value == LiteralType::Float(0.0)) {
                return Err(EvaluateError::DivisionByZero { dividend: l.clone(), divisor: r.clone() });
            }
            
            evaluate_binary(&left_value, o, &right_value)
        }
        Expression::Assignment { target: t, value: v } => {
            match &**t {
                Expression::Variable(name) => { 
                    let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;

                    let map: &mut HashMap<String, VariableData> = if let Some(m) = vars.iter_mut().rev().find(|map| map.contains_key(name)) {
                        m
                    }
                    else if global_vars.contains_key(name) {
                        global_vars
                    }
                    else {
                        return Err(EvaluateError::UndeclaredVariable(name.clone()));
                    };

                    if !is_value_type_valid(&value, &map.get(name).unwrap().data_type) {
                        return Err(EvaluateError::UnexpectedVariableValueType { expected: map.get(name).unwrap().data_type.clone(), recieved: value });
                    }
                    
                    map.get_mut(name).unwrap().value = value.clone();

                    Ok(value)
                },
                Expression::MemberAccess { class, member } => {
                    let evaluated: Result<LiteralType, EvaluateError> = evaluate_expression(class, global_vars, vars, funcs, classes);
                    if let Ok(LiteralType::Instance(data)) = evaluated {
                        let value: LiteralType = evaluate_expression(v, global_vars, vars, funcs, classes)?;

                        if !data.borrow().vars.contains_key(member) {
                            return Err(EvaluateError::UndeclaredVariableInClass { class: class.clone(), variable: member.clone() });
                        }
                        if !is_value_type_valid(&value, &data.borrow().vars.get(member).unwrap().data_type) {
                            return Err(EvaluateError::UnexpectedVariableValueType { expected: data.borrow().vars.get(member).unwrap().data_type.clone(), recieved: value });
                        }

                        data.borrow_mut().vars.get_mut(member).unwrap().value = value.clone();

                        Ok(value)
                    }
                    else {
                        Err(EvaluateError::ExpressionIsNotClass { expr: class.clone(), value: evaluated? })
                    }
                },
                _ => unreachable!() 
            }
        },
        Expression::Variable(n) => {
            if let Some(map) = vars.iter().rev().find(|map| map.contains_key(n)) {
                Ok(map.get(n).unwrap().value.clone())
            }
            else if global_vars.contains_key(n) {
                Ok(global_vars.get(n).unwrap().value.clone())
            }
            else {
                Err(EvaluateError::UndeclaredVariable(n.clone()))
            }
        },
        Expression::Call { callee, parameters } => {
            match &**callee {
                Expression::Variable(n) => {
                    if !funcs.contains_key(n) {
                        return Err(EvaluateError::UndeclaredFunction(n.clone()));
                    }

                    let mut func_vars: Vec<HashMap<String, VariableData>> = vec![HashMap::new()];
                    let func_data: FunctionData = funcs.get(n).unwrap().clone();

                    if parameters.len() != func_data.parameters.len() {
                        return Err(EvaluateError::UnexpectedParameterCount { callee: callee.clone(), expected: func_data.parameters.len(), got: parameters.len() });
                    }
                    for (i, (d, s)) in func_data.parameters.iter().enumerate() {
                        let p: LiteralType = match parameters.get(i) {
                            Some(v) => evaluate_expression(v, global_vars, vars, funcs, classes)?,
                            None => unreachable!(),
                        };
                        if !is_value_type_valid(&p, d) {
                            return Err(EvaluateError::UnexpectedParameterType { callee: callee.clone(), expected: d.clone(), got: p });
                        }

                        func_vars.first_mut().unwrap().insert(s.clone(), VariableData { data_type: d.clone(), value: p });
                    }

                    let return_value: LiteralType = match evaluate_statements(&func_data.block, global_vars, &mut func_vars, funcs, classes)? {
                        ControlFlow::None => LiteralType::Null,
                        ControlFlow::Return(expr) => expr.unwrap_or(LiteralType::Null),
                        ControlFlow::Continue => return Err(EvaluateError::ContinueStatementNotInLoop),
                    };

                    let correct_type: bool = match func_data.data_type.clone() {
                        Some(d) => is_value_type_valid(&return_value, &d),
                        None => return_value == LiteralType::Null,
                    };

                    if correct_type {
                        Ok(return_value)
                    }
                    else {
                        Err(EvaluateError::UnexpectedReturnValueType { expected: func_data.data_type, recieved: return_value })
                    }
                },
                Expression::MemberAccess { class, member } => {
                    if let Expression::Variable(name) = &**class {
                        if member == "new" && classes.contains_key(name) {
                            if parameters.len() != 0 {
                                return Err(EvaluateError::UnexpectedParameterCount { callee: callee.clone(), expected: 0, got: parameters.len() });
                            }

                            let mut instance: InstanceData = InstanceData { vars: HashMap::with_capacity(classes.get(name).unwrap().vars.len()), class: classes.get(name).unwrap().clone() };

                            let vars_iter: Vec<(String, (DataType, Box<Expression>))> = classes.get(name).unwrap().vars.iter().map(|(s, (d, e))| (s.clone(), (d.clone(), e.clone()))).collect::<Vec<(String, (DataType, Box<Expression>))>>();
                            for var in vars_iter {
                                let value: LiteralType = evaluate_expression(&var.1.1, global_vars, vars, funcs, classes)?;

                                if !is_value_type_valid(&value, &var.1.0) {
                                    return Err(EvaluateError::UnexpectedVariableValueType { expected: var.1.0, recieved: value });
                                }

                                instance.vars.insert(var.0, VariableData { data_type: var.1.0, value });   
                            }

                            return Ok(LiteralType::Instance(Rc::new(RefCell::new(instance))));
                        }
                    }
                    let evaluated: Result<LiteralType, EvaluateError> = evaluate_expression(&class, global_vars, vars, funcs, classes);
                    if let Ok(LiteralType::Instance(data)) = evaluated {
                        if !data.borrow().class.funcs.contains_key(member) {
                            return Err(EvaluateError::UndeclaredFunctionInClass { class: class.clone(), function: member.clone() });
                        }
                        
                        let mut func_vars: Vec<HashMap<String, VariableData>> = vec![HashMap::new()];
                        let func_data: FunctionData = data.borrow().class.funcs.get(member).unwrap().clone();

                        func_vars.first_mut().unwrap().insert(
                            "this".to_owned(),
                            VariableData {
                                data_type: DataType::Instance,
                                value: LiteralType::Instance(Rc::clone(&data))
                            }
                        );

                        if parameters.len() != func_data.parameters.len() {
                            return Err(EvaluateError::UnexpectedParameterCount { callee: callee.clone(), expected: func_data.parameters.len(), got: parameters.len() });
                        }
                        for (i, (d, s)) in func_data.parameters.iter().enumerate() {
                            let p: LiteralType = match parameters.get(i) {
                                Some(v) => evaluate_expression(v, global_vars, vars, funcs, classes)?,
                                None => unreachable!(),
                            };
                            if !is_value_type_valid(&p, d) {
                                return Err(EvaluateError::UnexpectedParameterType { callee: callee.clone(), expected: d.clone(), got: p });
                            }

                            func_vars.first_mut().unwrap().insert(s.clone(), VariableData { data_type: d.clone(), value: p });
                        }

                        let return_value: LiteralType = match evaluate_statements(&func_data.block, global_vars, &mut func_vars, funcs, classes)? {
                            ControlFlow::None => LiteralType::Null,
                            ControlFlow::Return(expr) => expr.unwrap_or(LiteralType::Null),
                            ControlFlow::Continue => return Err(EvaluateError::ContinueStatementNotInLoop),
                        };

                        let correct_type: bool = match func_data.data_type.clone() {
                            Some(d) => is_value_type_valid(&return_value, &d),
                            None => return_value == LiteralType::Null,
                        };

                        if correct_type {
                            Ok(return_value)
                        }
                        else {
                            Err(EvaluateError::UnexpectedReturnValueType { expected: func_data.data_type, recieved: return_value })
                        }
                    }
                    else {
                        Err(EvaluateError::ExpressionIsNotClass { expr: class.clone(), value: evaluated? })
                    }
                }
               other => Err(EvaluateError::UnexpectedFunctionCallee(other.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            let evaluated: Result<LiteralType, EvaluateError> = evaluate_expression(class, global_vars, vars, funcs, classes);
            if let Ok(LiteralType::Instance(data)) = evaluated {
                if !data.borrow().vars.contains_key(member) {
                    return Err(EvaluateError::UndeclaredVariableInClass { class: class.clone(), variable: member.clone() });
                }

                Ok(data.borrow().vars.get(member).unwrap().value.clone())
            }
            else {
                Err(EvaluateError::ExpressionIsNotClass { expr: class.clone(), value: evaluated? })
            }
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

        (LiteralType::Int(v1), BinaryOp::Equal, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 == v2)),
        (LiteralType::Float(v1), BinaryOp::Equal, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 == v2 as f64)),
        (t1, BinaryOp::Equal, t2) => Ok(LiteralType::Bool(t1 == t2)),

        (LiteralType::Int(v1), BinaryOp::NotEqual, LiteralType::Float(v2)) => Ok(LiteralType::Bool(v1 as f64 != v2)),
        (LiteralType::Float(v1), BinaryOp::NotEqual, LiteralType::Int(v2)) => Ok(LiteralType::Bool(v1 != v2 as f64)),
        (t1, BinaryOp::NotEqual, t2) => Ok(LiteralType::Bool(t1 != t2)),

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

fn evaluate_block(statements: &[Statement], global_vars: &mut HashMap<String, VariableData>, vars: &mut Vec<HashMap<String, VariableData>>, funcs: &mut HashMap<String, FunctionData>, classes: &mut HashMap<String, Rc<ClassData>>) -> Result<ControlFlow, EvaluateError> {
    vars.push(HashMap::new());
    let result: Result<ControlFlow, EvaluateError> = evaluate_statements(statements, global_vars, vars, funcs, classes);
    vars.pop();
    result
}

fn is_value_type_valid(value: &LiteralType, data_type: &DataType) -> bool {
    match (value, data_type) {
        (LiteralType::Null, DataType::Nullable(_)) => true,
        (v, DataType::Nullable(inner_type)) => is_value_type_valid(v, inner_type),

        (LiteralType::Int(_), DataType::Int) => true,
        (LiteralType::Float(_), DataType::Float) => true,
        (LiteralType::String(_), DataType::String) => true,
        (LiteralType::Bool(_), DataType::Bool) => true,
        (LiteralType::Instance(_), DataType::Instance) => true,

        _ => false,
    }
}