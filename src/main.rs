use crate::parser::{LiteralType, Statement};
use crate::lexer::Token;
use std::collections::HashMap;

mod lexer;
mod parser;
mod evaluator;

fn main() {
    let source: &'static str = "let int x = 27;";
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statement: Statement = parser::parse_tokens(tokens).unwrap();
    let mut vars: Vec<HashMap<String, LiteralType>> = vec![HashMap::new()];
    evaluator::evaluate_statment(statement, &mut vars).unwrap();
}
