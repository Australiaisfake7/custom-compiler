use crate::parser::{Expression, LiteralType};
use crate::lexer::Token;

mod lexer;
mod parser;
mod evaluator;

fn main() {
    let source: &str = "(3 + 5) / 2.5 <= 3.4";
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let expression: Box<Expression> = parser::parse_tokens(tokens).unwrap();
    let v: LiteralType = evaluator::evaluate(*expression).expect("Parsing failed");
    println!("{:?}", v);
}
