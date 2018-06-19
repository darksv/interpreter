use ::std::fmt;

#[derive(Clone, Copy)]
pub enum Inst {
    Ldarg(u8),
    Starg(u8),
    Add,
    Jump(u32),
    Beq(u32),
    Breakpoint,
    Call(u16),
    Ret,
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Inst::Add => write!(f, "add")?,
            &Inst::Jump(dst) => write!(f, "jump {}", dst)?,
            &Inst::Beq(dst) => write!(f, "beq {}", dst)?,
            &Inst::Ldarg(idx) => write!(f, "ldarg {}", idx)?,
            &Inst::Starg(idx) => write!(f, "starg {}", idx)?,
            &Inst::Breakpoint => write!(f, "breakpoint")?,
            &Inst::Ret => write!(f, "ret")?,
            &Inst::Call(idx) => write!(f, "call {}", idx)?,
        };
        Ok(())
    }
}
