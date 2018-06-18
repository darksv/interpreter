#[derive(Clone, Copy)]
enum Inst {
    Ldarg(u8),
    Starg(u8),
    Add,
    Jump(u32),
    Beq(u32),
    Breakpoint,
    Call(u16),
    Ret,
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
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

fn parse_operand<'a, T, I>(parts: &mut I) -> T
    where
        T: std::str::FromStr,
        I: Iterator<Item=&'a str> {
    parts
        .next()
        .and_then(|x| x.parse::<T>().ok())
        .unwrap()
}

struct CallFrame {
    inst_ptr: u32,
    stack: Vec<u32>,
    locals: Vec<u32>,
}

impl CallFrame {
    fn with_locals(locals: Vec<u32>) -> Self {
        Self {
            inst_ptr: 0,
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
}

fn main() {
    let mut loader = Loader::new();
    let asm = loader.load("input.asm");
    print_assembly(&asm);
    execute_assembly(&asm)
}

fn execute_assembly(asm: &Assembly) {
    let mut call_stack = vec![
        (&asm.functions[0], CallFrame::with_locals(vec![1, 2, 3, 4, 5])),
    ];
    while !call_stack.is_empty() {
        let result = {
            let (caller, caller_frame) = call_stack.last_mut().unwrap();
            match execute(caller, caller_frame) {
                ExecutionStatus::Call(func_idx) => {
                    let callee = &asm.functions[func_idx as usize];
                    let mut locals = vec![0; 5];
                    for idx in 0..callee.args {
                        locals[idx as usize] = caller_frame.pop().unwrap();
                    }
                    let callee_frame = CallFrame::with_locals(locals);
                    Some((callee, callee_frame))
                }
                ExecutionStatus::Return => {
                    None
                }
            }
        };

        match result {
            Some((callee, callee_frame)) => {
                {
                    let (caller, _caller_frame) = call_stack.last().unwrap();
                    eprintln!("Calling '{}' from '{}'", callee.name, caller.name);
                }
                call_stack.push((callee, callee_frame));
            },
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

#[derive(Clone)]
struct FuncDef {
    name: String,
    args: u16,
    returns: bool,
    body: Vec<Inst>,
}

struct Loader {
    functions: Vec<FuncDef>,
    pending_labels: Vec<String>,
    label_offsets: std::collections::HashMap<String, usize>,
    labels: Vec<String>,
    current_func: Option<FuncDef>,
}

struct Assembly {
    name: String,
    functions: Vec<FuncDef>,
}

fn print_assembly(asm: &Assembly) {
    println!("Assembly '{}':", &asm.name);
    for (idx, func) in asm.functions.iter().enumerate() {
        println!(" Function #{} '{}':", idx, func.name);
        for val in func.body.iter() {
            println!("  {}", val);
        }
    }
}

impl Loader {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            pending_labels: Vec::new(),
            label_offsets: std::collections::HashMap::new(),
            labels: Vec::new(),
            current_func: None,
        }
    }

    fn load(&mut self, path: &str) -> Assembly {
        use std::io::{BufReader, BufRead};
        use std::fs::File;

        let file = File::open(path).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('.') {
                self.process_meta(&line[1..]);
            } else if line.ends_with(':') {
                self.process_label(&line[..line.len() - 1]);
            } else {
                self.process_instruction(&line);
            }
        }

        Assembly {
            name: path.into(),
            functions: self.functions.clone(),
        }
    }

    fn adjust_branches(&mut self) {
        let mut changes = vec![];
        for (index, inst) in self.current_func.as_ref().unwrap().body.iter().enumerate() {
            let new_inst = match *inst {
                Inst::Jump(idx) => Inst::Jump(self.get_real_offset(idx)),
                Inst::Beq(idx) => Inst::Beq(self.get_real_offset(idx)),
                _ => continue,
            };
            changes.push((index, new_inst));
        }
        for (index, new_inst) in changes {
            self.current_func.as_mut().unwrap().body[index] = new_inst;
        }
    }

    fn get_real_offset(&self, idx: u32) -> u32 {
        self.label_offsets[&self.labels[idx as usize]] as u32
    }

    fn save_pending_labels(&mut self) {
        let position = self.current_func.as_ref().unwrap().body.len();
        for label in self.pending_labels.drain(..) {
            self.label_offsets.insert(label, position);
        }
    }

    fn process_meta(&mut self, line: &str) {
        if self.current_func.is_some() {
            self.save_pending_labels();
            self.adjust_branches();
        }

        if let Some(func) = self.current_func.take() {
            self.functions.push(func);
        }

        let mut parts = line.split(' ');
        if parts.next().unwrap() == "func" {
            self.current_func = Some(FuncDef {
                name: parts.next().unwrap().into(),
                args: parts.next().unwrap().parse().unwrap(),
                returns: parts.next().unwrap().parse().unwrap(),
                body: Vec::new(),
            });

            self.pending_labels.clear();
            self.labels.clear();
            self.label_offsets.clear();
        }
    }

    fn process_label(&mut self, label: &str) {
        self.pending_labels.push(label.to_owned());
    }

    fn process_instruction(&mut self, line: &str) {
        let mut parts = line.split(' ');
        let op = match parts.next().unwrap_or("") {
            "ldarg" => Inst::Ldarg(parse_operand(&mut parts)),
            "starg" => Inst::Starg(parse_operand(&mut parts)),
            "jump" => {
                let label = parts.next().unwrap();
                Inst::Jump(self.save_label(label) as u32)
            }
            "beq" => {
                let label = parts.next().unwrap();
                Inst::Beq(self.save_label(label) as u32)
            }
            "add" => Inst::Add,
            "breakpoint" => Inst::Breakpoint,
            "call" => Inst::Call(parse_operand(&mut parts)),
            "ret" => Inst::Ret,
            other => unreachable!("{}", other),
        };

        self.save_pending_labels();
        self.current_func.as_mut().unwrap().body.push(op);
    }

    fn save_label(&mut self, label: &str) -> usize {
        match self.labels.iter().position(|x| x == label) {
            Some(idx) => idx,
            None => {
                self.labels.push(label.to_owned());
                self.labels.len() - 1
            }
        }
    }
}

enum ExecutionStatus {
    Call(u16),
    Return,
}

fn execute(function: &FuncDef, frame: &mut CallFrame) -> ExecutionStatus {
    loop {
        if frame.inst_ptr as usize >= function.body.len() {
            println!("Exit.");
            break;
        }

        match function.body[frame.inst_ptr as usize] {
            Inst::Add => {
                let value2 = frame.pop().unwrap();
                let value1 = frame.pop().unwrap();
                frame.push(value2 + value1);
            }
            Inst::Jump(target) => {
                frame.inst_ptr = target;
                continue;
            }
            Inst::Beq(target) => {
                let value2 = frame.pop().unwrap();
                let value1 = frame.pop().unwrap();
                if value1 == value2 {
                    frame.inst_ptr = target;
                    continue;
                }
            }
            Inst::Ldarg(n) => {
                let value = frame.locals[n as usize];
                frame.push(value);
            }
            Inst::Starg(n) => {
                frame.locals[n as usize] = frame.pop().unwrap();
            }
            Inst::Call(idx) => {
                frame.inst_ptr += 1;
                return ExecutionStatus::Call(idx);
            }
            Inst::Ret => {
                frame.inst_ptr += 1;
                break;
            }
            Inst::Breakpoint => {
                print_debug_info(function, frame)
            },
        }
        frame.inst_ptr += 1;
    }

    ExecutionStatus::Return
}

fn print_debug_info(function: &FuncDef, frame: &CallFrame) -> () {
    println!("Code:");
    for (idx, val) in function.body.iter().enumerate() {
        let pc = frame.inst_ptr as usize;
        println!("{} [{:0>4}] {}", if idx == pc { ">" } else { " " }, idx, val);
    }
    println!("Locals:");
    for (idx, value) in frame.locals.iter().enumerate() {
        println!("  [{:0>4}] 0x{:0>8x}", idx, value);
    }
}
