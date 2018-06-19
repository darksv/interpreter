use super::instructions::Inst;

pub struct Assembly {
    pub name: String,
    pub functions: Vec<FuncDef>,
}

#[derive(Clone)]
pub struct FuncDef {
    pub name: String,
    pub args: u16,
    pub returns: bool,
    pub default_locals: Vec<u32>,
    pub body: Vec<Inst>,
}

pub fn print_assembly(asm: &Assembly) {
    println!("Assembly '{}':", &asm.name);
    for (idx, func) in asm.functions.iter().enumerate() {
        println!(" Function #{} '{}' - locals: {}:", idx, func.name, func.default_locals.len());
        for val in func.body.iter() {
            println!("  {}", val);
        }
    }
}
