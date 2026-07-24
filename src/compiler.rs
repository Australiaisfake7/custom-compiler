use parser::{Statement, ClassData, FunctionData, VariableData};
use lexer::Token;
use evaluator::ControlFlow;
use std::collections::HashMap;

mod lexer;
mod parser;
mod evaluator;

pub fn compile(source: &'static str) {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
    let mut global_vars: HashMap<String, VariableData> = HashMap::new();
    let mut vars: Vec<HashMap<String, VariableData>> = Vec::new();
    let mut funcs: HashMap<String, FunctionData> = HashMap::new();
    let mut classes: HashMap<String, ClassData> = HashMap::new();
    evaluator::evaluate_statements(&statements, &mut global_vars, &mut vars, &mut funcs, &mut classes).unwrap();
}
