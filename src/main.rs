mod compiler;

use compiler::compile;

fn main() {
    compile("let int x = 0; while (x < 10) {print x; x = x + 1; }");
}