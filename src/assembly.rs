use super::instructions::Inst;

pub struct Assembly {
    pub entry: u16,
    pub name: String,
    pub functions: Vec<FuncDef>,
}

impl Assembly {
    pub fn get_entry(&self) -> &FuncDef {
        &self.functions[self.entry as usize]
    }
}

#[derive(Clone)]
pub enum FuncDef {
    Managed(ManagedFuncDef),
    Native(NativeFuncDef),
}

impl FuncDef {
    pub fn name<'a>(&'a self) -> &'a str {
        match self {
            FuncDef::Managed(ManagedFuncDef { name, .. }) => &name,
            FuncDef::Native(NativeFuncDef { name, .. }) => &name,
        }
    }

    pub fn as_managed(&self) -> Option<&ManagedFuncDef> {
        match self {
            FuncDef::Managed(ref func) => Some(func),
            FuncDef::Native(_) => None,
        }
    }

    pub fn as_managed_mut(&mut self) -> Option<&mut ManagedFuncDef> {
        match self {
            FuncDef::Managed(ref mut func) => Some(func),
            FuncDef::Native(_) => None,
        }
    }
}

#[derive(Clone)]
pub struct ManagedFuncDef {
    pub name: String,
    pub args: u16,
    pub returns: bool,
    pub default_locals: Vec<u32>,
    pub body: Vec<Inst>,
}

#[derive(Clone)]
pub struct NativeFuncDef {
    pub name: String,
    pub args: u16,
    pub returns: bool,
}

pub fn print_assembly(asm: &Assembly) {
    println!("Assembly '{}' with entry point '{}':", &asm.name, asm.get_entry().name());
    for (idx, func) in asm.functions.iter().enumerate() {
        let func = func.as_managed().unwrap();
        println!(" Function #{} '{}' - locals: {}:", idx, func.name, func.default_locals.len());
        for val in func.body.iter() {
            println!("  {}", val);
        }
    }
}
