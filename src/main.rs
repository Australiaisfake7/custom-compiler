mod compiler;

use compiler::compile;

fn main() {
    compile("
    fun int printNum(int num;) {
        return num;
    }
    
    print printNum(2;);");
}