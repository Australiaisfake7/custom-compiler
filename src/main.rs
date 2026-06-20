mod compiler;

use compiler::compile;

fn main() {
    compile("for(let int i = 0; i < 10; i = i + 1;) {print i;}");
}