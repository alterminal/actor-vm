use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug)]
enum Value {
    Ref(u64),
    Int(i64),
    Float(f64),
    String(String),
    Atom(String),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Map(std::collections::HashMap<Value, Value>),
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Ref(r) => r.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Atom(a) => a.hash(state),
            Value::List(l) => {
                for item in l {
                    item.hash(state);
                }
            }
            Value::Tuple(t) => {
                for item in t {
                    item.hash(state);
                }
            }
            Value::Map(m) => {
                for (key, value) in m {
                    key.hash(state);
                    value.hash(state);
                }
            }
        }
    }
}

impl Value {
    fn clone(&self) -> Value {
        match self {
            Value::Ref(r) => Value::Ref(*r),
            Value::Int(i) => Value::Int(*i),
            Value::Float(f) => Value::Float(*f),
            Value::String(s) => Value::String(s.clone()),
            Value::Atom(a) => Value::Atom(a.clone()),
            Value::List(l) => {
                let mut new_list: Vec<Value> = Vec::new();
                for item in l {
                    new_list.push(item.clone());
                }
                Value::List(new_list)
            }
            Value::Tuple(t) => {
                let mut new_tuple: Vec<Value> = Vec::new();
                for item in t {
                    new_tuple.push(item.clone());
                }
                Value::Tuple(new_tuple)
            }
            Value::Map(m) => {
                let mut new_map: std::collections::HashMap<Value, Value> =
                    std::collections::HashMap::new();
                Value::Map(new_map)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Reg {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    ZF,
    LR,
    RegCount,
}

enum Inst {
    Int(Reg, i64),
    Float(Reg, f64),
    Ref(Reg, u64),
    String(Reg, String),
    Atom(Reg, String),
    List(Reg),
    Tuple(Reg),
    Map(Reg),
    Move(Reg, Reg),
    Store(Reg, u64),
    Load(u64, Reg),
    Send(Reg, Reg),
    Recv(Reg),
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Mul(Reg, Reg, Reg),
    Div(Reg, Reg, Reg),
    Mod(Reg, Reg, Reg),
    Jump(u64),
    JumpIf(u64),
    Eq(Reg, Reg),
    Ne(Reg, Reg),
    Gt(Reg, Reg),
    Lt(Reg, Reg),
    Gte(Reg, Reg),
    Lte(Reg, Reg),
    Push(Reg),
    Pop(Reg),
    Hlt,
}

struct Mailbox {
    messages: Vec<Value>,
    lock: std::sync::Mutex<()>,
}

impl Mailbox {
    fn post(&mut self, value: Value) {
        let _lock = self.lock.lock().unwrap();
        self.messages.push(value);
    }

    fn take(&mut self) -> Option<Value> {
        let _lock = self.lock.lock().unwrap();
        if self.messages.is_empty() {
            None
        } else {
            Some(self.messages.remove(0))
        }
    }

    fn new() -> Mailbox {
        Mailbox {
            messages: Vec::new(),
            lock: std::sync::Mutex::new(()),
        }
    }
}

struct Register {
    registers: [Value; 11],
}

impl Register {
    fn show_reg(&self) {
        for (i, reg) in self.registers.iter().enumerate() {
            println!("R{}: {:?}", i, reg);
        }
    }
    fn get(&self, reg: Reg) -> &Value {
        &self.registers[reg as usize]
    }
    fn set(&mut self, reg: Reg, value: &Value) {
        self.registers[reg as usize] = value.clone();
    }
    fn new() -> Register {
        Register {
            registers: [
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
                Value::Ref(0),
            ],
        }
    }
}

struct ActorVm {
    register: Register,
    clock_count: u64,
    cpu: u64,
    stack: Vec<Value>,
    heap: Vec<Value>,
    mailbox: Mailbox,
    lock: std::sync::Mutex<()>,
    program: Vec<Inst>,
    sender: fn(Value, Value),
}

impl ActorVm {
    fn get_tick(&mut self) -> bool {
        self.lock.try_lock().is_ok()
    }

    fn release(&mut self) {}

    fn pc(&self) -> u64 {
        let pc = self.register.get(Reg::PC);
        match pc {
            Value::Ref(r) => *r,
            _ => panic!("PC is not a reference"),
        }
    }

    fn set_pc(&mut self, pc: u64) {
        self.register.set(Reg::PC, &Value::Ref(pc));
    }

    fn get_reg(&self, reg: Reg) -> &Value {
        &self.register.get(reg)
    }

    fn set_reg(&mut self, reg: Reg, value: &Value) {
        self.register.set(reg, value);
    }

    fn tick(&mut self) {
        let pc = self.pc();
        let inst: &Inst = &self.program[pc as usize];
        self.register.set(Reg::PC, &Value::Ref(pc + 1));
        match *inst {
            Inst::Int(reg, v) => {
                self.set_reg(reg, &Value::Int(v));
            }
            Inst::Float(reg, v) => {
                self.set_reg(reg, &Value::Float(v));
            }
            Inst::Ref(reg, v) => {
                self.set_reg(reg, &Value::Ref(v));
            }
            Inst::String(reg, ref s) => {
                self.set_reg(reg, &Value::String(s.clone()));
            }
            Inst::Atom(reg, ref s) => {
                self.set_reg(reg, &Value::Atom(s.clone()));
            }
            Inst::List(reg) => {
                let list = Value::List(vec![]);
                self.set_reg(reg, &list);
            }
            Inst::Move(r1, r2) => {
                let value = self.get_reg(r1);
                self.set_reg(r2, &value.clone());
            }
            Inst::Add(r0, r1, r2) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(r2, &Value::Int(v0 + v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(r2, &Value::Float(v0 + v1));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn post(&mut self, value: Value) {
        self.mailbox.post(value);
    }

    fn show_reg(&self) {
        self.register.show_reg();
    }

    fn new(program: Vec<Inst>, sender: fn(Value, Value), cpu: u64) -> ActorVm {
        ActorVm {
            clock_count: cpu,
            cpu: cpu,
            register: Register::new(),
            stack: Vec::new(),
            heap: Vec::new(),
            mailbox: Mailbox::new(),
            lock: std::sync::Mutex::new(()),
            program: program,
            sender: sender,
        }
    }
}

fn sender(value: Value, to: Value) {
    // Implement the sender function
    println!("Sender function called, value: {:?}, to: {:?}", value, to);
}

fn main() {
    let mut actor = ActorVm::new(
        vec![
            Inst::Int(Reg::R1, 123),
            Inst::Move(Reg::R1, Reg::R0),
            Inst::Add(Reg::R0, Reg::R1, Reg::R2),
            Inst::Send(Reg::R2, Reg::R0),
            Inst::Hlt,
        ],
        sender,
        1000,
    );
    actor.tick();
    actor.tick();
    actor.tick();
    actor.show_reg();
}
