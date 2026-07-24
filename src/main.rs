mod compiler;

use compiler::compile;

fn main() {
    compile("
    class N {
        let int v;
        fun printNum(string s;)
        {
            print s;
        }
    }
    let instance n = N.new();
    
    n.printNum(\"test\";);");
}