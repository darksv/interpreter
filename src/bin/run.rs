extern crate vm;

use std::env::args;
use vm::assembly::print_assembly;
use vm::loader::Loader;
use vm::interpreter::execute_assembly;

fn main() {
    let path = args().nth(1).unwrap();
    let mut loader = Loader::new();
    let asm = loader.load(&path);
    print_assembly(&asm);
    execute_assembly(&asm)
}
