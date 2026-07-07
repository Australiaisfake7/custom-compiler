mod compiler;

use compiler::compile;

fn main() {
    compile("
    class n {
        let int v;
        fun int f(int i;) {}
    }");
}