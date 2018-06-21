mod loader;
use loader::Loader;
mod interpreter;
use interpreter::execute_assembly;
mod instructions;
mod assembly;
use assembly::print_assembly;

fn main() {
    let mut loader = Loader::new();
    let asm = loader.load("tests/input.asm");
    print_assembly(&asm);
    execute_assembly(&asm)
}
