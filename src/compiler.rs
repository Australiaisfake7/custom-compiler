use parser::{LiteralType, Statement};
use lexer::Token;
use std::collections::HashMap;

mod lexer;
mod parser;
mod evaluator;

pub fn compile(source: &'static str) {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
    let mut vars: Vec<HashMap<String, LiteralType>> = vec![HashMap::new()];
    evaluator::evaluate_statements(&statements, &mut vars).unwrap();
}
