use parser::{LiteralType, Statement};
use lexer::Token;
use evaluator::ControlFlow;
use std::collections::HashMap;

use crate::compiler::parser::FunctionData;

mod lexer;
mod parser;
mod evaluator;

pub fn compile(source: &'static str) {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
    let mut global_vars: HashMap<String, LiteralType> = HashMap::new();
    let mut vars: Vec<HashMap<String, LiteralType>> = vec![HashMap::new()];
    let mut funcs: HashMap<String, FunctionData> = HashMap::new();
    match evaluator::evaluate_statements(&statements, &mut global_vars, &mut vars, &mut funcs).unwrap() {
        ControlFlow::None => (),
        ControlFlow::Return(_) => panic!("Unexpected return statement in global scope"),
    }
}
