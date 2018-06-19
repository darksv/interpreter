use super::instructions::Inst;
use super::assembly::{Assembly, FuncDef};

struct CallFrame {
    program_counter: u32,
    stack: Vec<u32>,
    locals: Vec<u32>,
}

impl CallFrame {
    fn with_locals(locals: Vec<u32>) -> Self {
        Self {
            program_counter: 0,
            stack: vec![],
            locals,
        }
    }

    fn push(&mut self, value: u32) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<u32> {
        self.stack.pop()
    }

    fn create_frame_for_callee(&mut self, callee: &FuncDef) -> CallFrame {
        let mut locals = callee.default_locals.clone();
        for idx in 0..callee.args {
            locals[idx as usize] = self.pop().unwrap();
        }
        CallFrame::with_locals(locals)
    }
}

enum ExecutionStatus {
    Call(u16),
    Return,
    Normal,
    Breakpoint,
}

pub fn execute_assembly(asm: &Assembly) {
    let main = asm.functions.first().unwrap();
    let mut call_stack = vec![
        (main, CallFrame::with_locals(main.default_locals.clone())),
    ];
    while !call_stack.is_empty() {
        let callee = {
            let (caller, ref mut caller_frame) = call_stack.last_mut().unwrap();
            loop {
                match step(caller, caller_frame) {
                    ExecutionStatus::Normal => (),
                    ExecutionStatus::Call(callee_idx) => {
                        let callee = &asm.functions[callee_idx as usize];
                        let callee_frame = caller_frame.create_frame_for_callee(callee);
                        break Some((callee, callee_frame));
                    }
                    ExecutionStatus::Return => break None,
                    ExecutionStatus::Breakpoint => print_debug_info(caller, caller_frame),
                }
            }
        };

        match callee {
            Some((callee, callee_frame)) => {
                {
                    let (caller, _caller_frame) = call_stack.last().unwrap();
                    eprintln!("Calling '{}' from '{}'", callee.name, caller.name);
                }
                call_stack.push((callee, callee_frame));
            }
            None => {
                let (callee, callee_frame) = call_stack.pop().unwrap();
                if callee.returns {
                    let result = callee_frame.locals[callee.args as usize];
                    if let Some((_, ref mut caller_frame)) = call_stack.last_mut() {
                        caller_frame.push(result);
                    }
                    eprintln!("Returning from '{}' with result '{}'", callee.name, result);
                } else {
                    eprintln!("Returning from '{}'", callee.name);
                }
            }
        }
    }
}

macro_rules! binary_op {
    ($frame:expr, $op:expr) => {{
        let value2 = $frame.pop().unwrap() as _;
        let value1 = $frame.pop().unwrap() as _;
        $frame.push($op(value2, value1) as u32);
    }}
}

fn step(function: &FuncDef, frame: &mut CallFrame) -> ExecutionStatus {
    if frame.program_counter as usize >= function.body.len() {
        return ExecutionStatus::Return;
    }
    match function.body[frame.program_counter as usize] {
        Inst::add_u => binary_op!(frame, |a: u32, b: u32| a + b),
        Inst::add_s => binary_op!(frame, |a: i32, b: i32| a + b),
        Inst::sub_u => binary_op!(frame, |a: u32, b: u32| a - b),
        Inst::sub_s => binary_op!(frame, |a: i32, b: i32| a - b),
        Inst::jump(target) => {
            frame.program_counter = target;
            return ExecutionStatus::Normal;
        }
        Inst::beq(target) => {
            let value2 = frame.pop().unwrap();
            let value1 = frame.pop().unwrap();
            if value1 == value2 {
                frame.program_counter = target;
                return ExecutionStatus::Normal;
            }
        }
        Inst::ldarg(n) => {
            let value = frame.locals[n as usize];
            frame.push(value);
        }
        Inst::starg(n) => {
            frame.locals[n as usize] = frame.pop().unwrap();
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



fn print_debug_info(function: &FuncDef, frame: &CallFrame) {
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
