use ::std::fmt;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Inst {
    ldarg(u8),
    starg(u8),
    add_u,
    add_s,
    sub_u,
    sub_s,
    mul_u,
    mul_s,
    div_u,
    div_s,
    jump(u32),
    beq(u32),
    breakpoint,
    call(u16),
    ret,
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Inst::add_u => write!(f, "add.u")?,
            &Inst::add_s => write!(f, "add.s")?,
            &Inst::sub_u => write!(f, "sub.u")?,
            &Inst::sub_s => write!(f, "sub.s")?,
            &Inst::mul_u => write!(f, "mul.u")?,
            &Inst::mul_s => write!(f, "mul.s")?,
            &Inst::div_u => write!(f, "div.u")?,
            &Inst::div_s => write!(f, "div.s")?,
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
