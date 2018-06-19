use ::std::collections::HashMap;
use ::std::str::FromStr;
use super::instructions::Inst;
use super::assembly::{Assembly, FuncDef};

pub struct Loader {
    functions: Vec<FuncDef>,
    pending_labels: Vec<String>,
    label_offsets: HashMap<String, usize>,
    labels: Vec<String>,
    current_func: Option<FuncDef>,
    called_names: Vec<String>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            pending_labels: Vec::new(),
            label_offsets: HashMap::new(),
            labels: Vec::new(),
            current_func: None,
            called_names: Vec::new(),
        }
    }

    pub fn load(&mut self, path: &str) -> Assembly {
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
                self.parse_label(&line[..line.len() - 1]);
            } else {
                self.parse_instruction(&line);
            }
        }
        self.save_func();
        self.fill_call_placeholders();
        Assembly {
            name: path.into(),
            functions: self.functions.clone(),
        }
    }

    fn fill_call_placeholders(&mut self) {
        let mut changes = vec![];
        for (caller_idx, caller) in self.functions.iter().enumerate() {
            for (inst_idx, inst) in caller.body.iter().enumerate() {
                if let Inst::call(fake_idx) = inst {
                    let real_idx = self.get_real_func_index(*fake_idx);
                    changes.push((caller_idx, inst_idx, real_idx))
                }
            }
        }

        for (caller_idx, inst_idx, real_callee_idx) in changes {
            if let Inst::call(ref mut callee_idx) = self.functions[caller_idx].body[inst_idx] {
                *callee_idx = real_callee_idx;
            }
        }
    }

    fn get_real_func_index(&self, fake_idx: u16) -> u16 {
        let callee_name = &self.called_names[fake_idx as usize];
        self.functions.iter()
            .position(|x| &x.name == callee_name)
            .map(|idx| idx as u16)
            .expect("no such func")
    }

    fn fill_branching_placeholders(&mut self) {
        let mut changes = vec![];
        for (index, inst) in self.current_func.as_ref().unwrap().body.iter().enumerate() {
            let new_inst = match *inst {
                Inst::jump(idx) => Inst::jump(self.get_real_instruction_offset(idx)),
                Inst::beq(idx) => Inst::beq(self.get_real_instruction_offset(idx)),
                _ => continue,
            };
            changes.push((index, new_inst));
        }
        for (index, new_inst) in changes {
            self.current_func.as_mut().unwrap().body[index] = new_inst;
        }
    }

    fn get_real_instruction_offset(&self, idx: u32) -> u32 {
        self.label_offsets[&self.labels[idx as usize]] as u32
    }

    fn save_pending_labels(&mut self) {
        let position = self.current_func.as_ref().unwrap().body.len();
        for label in self.pending_labels.drain(..) {
            self.label_offsets.insert(label, position);
        }
    }

    fn process_meta(&mut self, line: &str) {
        let mut parts = line.split(' ');
        match parts.next().unwrap() {
            "func" => {
                self.save_func();

                let name = parts.next().unwrap().into();
                let args = parts.next().unwrap().parse().unwrap();
                let returns = parts.next().unwrap().parse().unwrap();

                self.current_func = Some(FuncDef {
                    name,
                    args,
                    returns,
                    body: Vec::new(),
                    default_locals: Vec::new(),
                });

                self.pending_labels.clear();
                self.labels.clear();
                self.label_offsets.clear();
            }
            "locals" => {
                if let Some(ref mut func) = self.current_func {
                    let count = parts.next().unwrap().parse().unwrap();
                    func.default_locals = vec![0; count];
                }
            }
            "local" => {
                let idx: u16 = parts.next().unwrap().parse().unwrap();
                let value = parts.next().unwrap().parse().unwrap();
                if let Some(ref mut func) = self.current_func {
                    func.default_locals[idx as usize] = value;
                }
            }
            unknown => eprintln!("unknown meta: '{}'", unknown)
        }
    }

    fn save_func(&mut self) {
        if self.current_func.is_some() {
            self.save_pending_labels();
            self.fill_branching_placeholders();
        }
        if let Some(mut func) = self.current_func.take() {
            let default_locals = func.args + if func.returns { 1 } else { 0 };
            if (default_locals as usize) > func.default_locals.len() {
                func.default_locals.resize(default_locals as usize, 0);
            }
            self.functions.push(func);
        }
    }

    fn parse_label(&mut self, label: &str) {
        self.pending_labels.push(label.to_owned());
    }

    fn parse_instruction(&mut self, line: &str) {
        let mut parts = line.split(' ');
        let op = match parts.next().unwrap_or("") {
            "ldarg" => Inst::ldarg(parse_operand(&mut parts)),
            "starg" => Inst::starg(parse_operand(&mut parts)),
            "jump" => {
                let label = parts.next().unwrap();
                Inst::jump(self.get_placeholder_for_label(label) as u32)
            }
            "beq" => {
                let label = parts.next().unwrap();
                Inst::beq(self.get_placeholder_for_label(label) as u32)
            }
            "add.u" => Inst::add_u,
            "add.s" => Inst::add_s,
            "sub.u" => Inst::sub_u,
            "sub.s" => Inst::sub_s,
            "breakpoint" => Inst::breakpoint,
            "call" => {
                let func_name = parts.next().unwrap();
                Inst::call(self.get_placeholder_for_func(func_name) as u16)
            },
            "ret" => Inst::ret,
            other => unreachable!("{}", other),
        };

        self.save_pending_labels();
        self.current_func.as_mut().unwrap().body.push(op);
    }

    fn get_placeholder_for_func(&mut self, name: &str) -> usize {
        get_index_for(&mut self.called_names, name)
    }

    fn get_placeholder_for_label(&mut self, label: &str) -> usize {
        get_index_for(&mut self.labels, label)
    }
}

fn get_index_for(set: &mut Vec<String>, value: &str) -> usize {
    match set.iter().position(|x| x == value) {
        Some(idx) => idx,
        None => {
            set.push(value.to_owned());
            set.len() - 1
        }
    }
}

fn parse_operand<'a, T, I>(parts: &mut I) -> T
    where
        T: FromStr,
        I: Iterator<Item=&'a str> {
    parts
        .next()
        .and_then(|x| x.parse::<T>().ok())
        .unwrap()
}
