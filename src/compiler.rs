use parser::{LiteralType, Statement};
use lexer::Token;
use std::collections::HashMap;

use crate::compiler::parser::FunctionData;

mod lexer;
mod parser;
mod evaluator;

pub fn compile(source: &'static str) {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
    let mut vars: Vec<HashMap<String, LiteralType>> = vec![HashMap::new()];
    let mut funcs: Vec<HashMap<String, FunctionData>> = vec![HashMap::new()];
    evaluator::evaluate_statements(&statements, &mut vars, &mut funcs).unwrap();
}
