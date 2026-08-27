use parser::{Statement, FunctionData, VariableData};
use lexer::Token;
use std::{collections::HashMap, rc::Rc};

mod lexer;
mod parser;
mod ast_flattener;

pub fn compile(source: &'static str) {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
}
