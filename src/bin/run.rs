extern crate vm;

use vm::assembly::print_assembly;
use vm::loader::Loader;
use vm::interpreter::execute_assembly;

fn main() {
    let mut loader = Loader::new();
    let asm = loader.load("tests/input.asm");
    print_assembly(&asm);
    execute_assembly(&asm)
}
