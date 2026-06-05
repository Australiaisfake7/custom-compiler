mod lexer;

use lexer::Scanner;

fn main() {
    let source: String = "test 123
    if . ;".to_string();

    let mut scanner: Scanner = Scanner::from_source(source.chars());

    println!("{:#?}", scanner.read_all_tokens());
}
