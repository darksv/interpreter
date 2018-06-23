use super::instructions::Inst;
use super::assembly::{Assembly, FuncDef, ManagedFuncDef};

struct ManagedCallFrame {
    program_counter: u32,
    stack: Vec<u32>,
    locals: Vec<u32>,
}

enum CallFrame {
    Managed(ManagedCallFrame),
    Native,
}

impl ManagedCallFrame {
    fn with_locals(locals: Vec<u32>) -> Self {
        ManagedCallFrame {
            program_counter: 0,
            stack: vec![],
            locals,
        }
    }

    fn create_frame_for_callee(&mut self, callee: &ManagedFuncDef) -> ManagedCallFrame {
        let mut locals = callee.default_locals.clone();
        for idx in 0..callee.args {
            locals[idx as usize] = self.stack.pop().unwrap();
        }
        Self::with_locals(locals)
    }
}

impl CallFrame {
    fn by_func(def: &FuncDef) -> CallFrame {
        match def {
            FuncDef::Managed(def) => CallFrame::Managed(ManagedCallFrame::with_locals(def.default_locals.clone())),
            FuncDef::Native(_def) => CallFrame::Native,
        }
    }

    fn as_managed(&self) -> Option<&ManagedCallFrame> {
        match *self {
            CallFrame::Managed(ref frame) => Some(frame),
            CallFrame::Native => None,
        }
    }

    fn as_managed_mut(&mut self) -> Option<&mut ManagedCallFrame> {
        match *self {
            CallFrame::Managed(ref mut frame) => Some(frame),
            CallFrame::Native => None,
        }
    }
}

enum ExecutionStatus {
    Call(u16),
    Return,
    Normal,
    Breakpoint,
}

pub fn execute_assembly(asm: &Assembly) {
    let entry = asm.get_entry();
    let mut call_stack = vec![
        (entry, CallFrame::by_func(entry)),
    ];
    while !call_stack.is_empty() {
        let callee = {
            let (caller, ref mut caller_frame) = call_stack.last_mut().unwrap();
            loop {
                let caller = caller.as_managed().unwrap();
                let caller_frame = caller_frame.as_managed_mut().unwrap();
                match step_managed(caller, caller_frame) {
                    ExecutionStatus::Normal => (),
                    ExecutionStatus::Call(callee_idx) => {
                        let callee = &asm.functions[callee_idx as usize];
                        let callee_frame = caller_frame.create_frame_for_callee(callee.as_managed().unwrap());
                        break Some((callee, callee_frame));
                    }
                    ExecutionStatus::Return => break None,
                    ExecutionStatus::Breakpoint => print_managed_debug_info(caller, caller_frame),
                }
            }
        };

        match callee {
            Some((callee, callee_frame)) => {
                {
                    let (caller, _caller_frame) = call_stack.last().unwrap();
                    eprintln!("Calling '{}' from '{}'", callee.name(), caller.name());
                }
                call_stack.push((callee, CallFrame::Managed(callee_frame)));
            }
            None => {
                let (callee, callee_frame) = call_stack.pop().unwrap();
                let callee = callee.as_managed().unwrap();
                let callee_frame = callee_frame.as_managed().unwrap();
                if callee.returns {
                    let result = callee_frame.locals[callee.args as usize];
                    if let Some((_, ref mut caller_frame)) = call_stack.last_mut() {
                        let caller_frame = caller_frame.as_managed_mut().unwrap();
                        caller_frame.stack.push(result);
                    }
                    eprintln!("Returning from '{}' with result '{}'", callee.name, result);
                } else {
                    eprintln!("Returning from '{}'", callee.name);
                }
            }
        }
    }
}

use num::cast::{FromPrimitive, ToPrimitive};

#[inline(always)]
fn binary<T>(frame: &mut ManagedCallFrame, operator: fn(T, T) -> T)
    where T: ToPrimitive + FromPrimitive
{
    let value2 = frame.stack.pop().and_then(FromPrimitive::from_u32).unwrap();
    let value1 = frame.stack.pop().and_then(FromPrimitive::from_u32).unwrap();
    let result = operator(value2, value1).to_u32().unwrap();
    frame.stack.push(result);
}

fn step_managed(function: &ManagedFuncDef, frame: &mut ManagedCallFrame) -> ExecutionStatus {
    if frame.program_counter as usize >= function.body.len() {
        return ExecutionStatus::Return;
    }
    match function.body[frame.program_counter as usize] {
        Inst::add_u => binary::<u32>(frame, |a, b| a + b),
        Inst::add_s => binary::<i32>(frame, |a, b| a + b),
        Inst::sub_u => binary::<u32>(frame, |a, b| a - b),
        Inst::sub_s => binary::<i32>(frame, |a, b| a - b),
        Inst::mul_u => binary::<u32>(frame, |a, b| a * b),
        Inst::mul_s => binary::<i32>(frame, |a, b| a * b),
        Inst::div_u => binary::<u32>(frame, |a, b| a / b),
        Inst::div_s => binary::<i32>(frame, |a, b| a / b),
        Inst::jump(target) => {
            frame.program_counter = target;
            return ExecutionStatus::Normal;
        }
        Inst::beq(target) => {
            let value2 = frame.stack.pop().unwrap();
            let value1 = frame.stack.pop().unwrap();
            if value1 == value2 {
                frame.program_counter = target;
                return ExecutionStatus::Normal;
            }
        }
        Inst::ldarg(n) => {
            let value = frame.locals[n as usize];
            frame.stack.push(value);
        }
        Inst::starg(n) => {
            frame.locals[n as usize] = frame.stack.pop().unwrap();
        }
        Inst::call(idx) => {
            frame.program_counter += 1;
            return ExecutionStatus::Call(idx);
        }
        Inst::ret => {
            frame.program_counter += 1;
            return ExecutionStatus::Return;
        }
        Inst::breakpoint => {
            frame.program_counter += 1;
            return ExecutionStatus::Breakpoint;
        }
    }
    frame.program_counter += 1;
    ExecutionStatus::Normal
}

fn print_managed_debug_info(function: &ManagedFuncDef, frame: &ManagedCallFrame) {
    println!("Code:");
    for (idx, val) in function.body.iter().enumerate() {
        let pc = frame.program_counter as usize;
        println!("{} [{:0>4}] {}", if idx == pc { ">" } else { " " }, idx, val);
    }

    println!("Stack:");
    for (idx, value) in frame.stack.iter().enumerate() {
        println!("  [{:0>4}] 0x{:0>8x}", frame.stack.len() - idx - 1, value);
    }

    println!("Locals:");
    for (idx, value) in frame.locals.iter().enumerate() {
        println!("  [{:0>4}] 0x{:0>8x}", idx, value);
    }
}
