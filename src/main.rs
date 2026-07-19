mod compiler;

use compiler::compile;

fn main() {
    compile("
    class n {
        let int v;
    }");
}