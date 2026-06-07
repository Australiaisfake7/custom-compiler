use crate::parser::Expression;
use crate::lexer::Token;

mod parser;
mod lexer;

fn main() {
    let source: &str = "(3 + 5) / 2.5 == 8";
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let expressions: Vec<Box<Expression>> = parser::parse_tokens(tokens).unwrap();
    println!("{:#?}", expressions);
}
