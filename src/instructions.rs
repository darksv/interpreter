use ::std::fmt;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Inst {
    ldarg(u8),
    starg(u8),
    add,
    jump(u32),
    beq(u32),
    breakpoint,
    call(u16),
    ret,
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Inst::add => write!(f, "add")?,
            &Inst::jump(dst) => write!(f, "jump {}", dst)?,
            &Inst::beq(dst) => write!(f, "beq {}", dst)?,
            &Inst::ldarg(idx) => write!(f, "ldarg {}", idx)?,
            &Inst::starg(idx) => write!(f, "starg {}", idx)?,
            &Inst::breakpoint => write!(f, "breakpoint")?,
            &Inst::ret => write!(f, "ret")?,
            &Inst::call(idx) => write!(f, "call {}", idx)?,
        };
        Ok(())
    }
}
