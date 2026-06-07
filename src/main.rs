use crate::parser::{Expression, Parser};
use crate::lexer::{Scanner, Token};

mod parser;
mod lexer;

fn main() {
    let source: &str = "(3 + 5) / 2.5 == 8";
    let mut scanner: Scanner = Scanner::from_source(source.chars());

    let tokens: Vec<Token> = scanner.read_all_tokens().unwrap();
    let mut parser: Parser = Parser::from_tokens(tokens);
    let expressions: Vec<Box<Expression>> = parser.parse_tokens().expect("Failed parsing with error");
    println!("{:#?}", expressions);
}
