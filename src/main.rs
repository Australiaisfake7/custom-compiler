mod compiler;

use compiler::compile;

fn main() {
    compile("print 5 > 6 && true;");
}