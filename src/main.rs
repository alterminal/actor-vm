use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;

#[derive(Debug)]
enum Value {
    Ref(usize),
    Int(i64),
    Float(f64),
    Bool(bool),
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
            Value::Bool(b) => b.hash(state),
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
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::String(s.clone()),
            Value::Atom(a) => Value::Atom(a.clone()),
            Value::List(l) => {
                let mut new_list: Vec<Value> = Vec::with_capacity(l.len());
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
                let new_map: std::collections::HashMap<Value, Value> =
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
    Bool(Reg, bool),
    Ref(Reg, usize),
    String(Reg, String),
    Atom(Reg, String),
    List(Reg, usize),
    Tuple(Reg, usize),
    Map(Reg),
    SetC(Reg, Reg, Reg),
    MoveC(Reg, Reg, Reg),
    Move(Reg, Reg),
    Store(Reg, usize),
    Load(usize, Reg),
    Send(Reg, Reg),
    Recv(Reg),
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Mul(Reg, Reg, Reg),
    Div(Reg, Reg, Reg),
    Mod(Reg, Reg, Reg),
    Jump(usize),
    JumpIf(usize),
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
        for i in 0..8 {
            println!("R{}: {:?}", i, self.registers[i]);
        }
        println!("PC: {:?}", self.registers[8]);
        println!("ZF: {:?}", self.registers[9]);
        println!("LR: {:?}", self.registers[10]);
    }
    fn get(&self, reg: Reg) -> Value {
        self.registers[reg as usize].clone()
    }

    fn set(&mut self, reg: Reg, value: &Value) {
        self.registers[reg as usize] = value.clone()
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
                Value::Bool(false),
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
    running: bool,
}

impl ActorVm {
    fn get_tick(&mut self) -> bool {
        self.lock.try_lock().is_ok()
    }

    fn release(&mut self) {}

    fn pc(&self) -> usize {
        let pc = self.register.get(Reg::PC);
        match pc {
            Value::Ref(r) => r,
            _ => panic!("PC is not a reference"),
        }
    }

    fn set_pc(&mut self, pc: usize) {
        self.register.set(Reg::PC, &Value::Ref(pc));
    }

    fn get_reg(&self, reg: Reg) -> Value {
        self.register.get(reg)
    }

    fn set_reg(&mut self, reg: Reg, value: &Value) {
        self.register.set(reg, value);
    }

    fn tick(&mut self) {
        let pc = self.pc();
        let inst: &Inst = &self.program[pc];
        self.register.set(Reg::PC, &Value::Ref(pc + 1));
        match *inst {
            Inst::Load(address, reg) => {
                let value = self.heap[address].clone();
                self.set_reg(reg, &value);
            }
            Inst::Store(reg, address) => {
                let value = self.get_reg(reg).clone();
                self.heap[address] = value;
            }
            Inst::Bool(reg, v) => {
                self.set_reg(reg, &Value::Bool(v));
            }
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
            Inst::Tuple(reg, size) => {
                let mut tuple = Vec::with_capacity(size);
                for _ in 1..size {
                    tuple.push(Value::Ref(0));
                }
                self.set_reg(reg, &Value::Tuple(tuple));
            }
            Inst::List(reg, size) => {
                let mut list = Vec::with_capacity(size);
                for _ in 1..size {
                    list.push(Value::Ref(0));
                }
                self.set_reg(reg, &Value::List(list));
            }
            Inst::Map(reg) => {
                let map = Value::Map(std::collections::HashMap::new());
                self.set_reg(reg, &map);
            }
            Inst::SetC(target, key, value) => {
                let t = self.get_reg(target);
                let k = self.get_reg(key);
                let v = self.get_reg(value);
                match t {
                    Value::List(mut list) => match k {
                        Value::Int(i) => {
                            list[i as usize] = v.clone();
                            self.set_reg(target, &Value::List(list));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Inst::MoveC(from, key, to) => {
                let f = self.get_reg(from);
                let k = self.get_reg(key);
                match f {
                    Value::List(list) => match k {
                        Value::Int(i) => {
                            let value = list[i as usize].clone();
                            self.set_reg(to, &value);
                        }
                        _ => {}
                    },
                    _ => {}
                }
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
            Inst::Sub(r0, r1, r2) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(r2, &Value::Int(v0 - v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(r2, &Value::Float(v0 - v1));
                    }
                    _ => {}
                }
            }
            Inst::Mul(r0, r1, r2) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(r2, &Value::Int(v0 * v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(r2, &Value::Float(v0 * v1));
                    }
                    _ => {}
                }
            }
            Inst::Div(r0, r1, r2) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(r2, &Value::Int(v0 / v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(r2, &Value::Float(v0 / v1));
                    }
                    _ => {}
                }
            }
            Inst::Mod(r0, r1, r2) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(r2, &Value::Int(v0 % v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(r2, &Value::Float(v0 % v1));
                    }
                    _ => {}
                }
            }
            Inst::Eq(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 == v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 == v1));
                    }
                    (Value::String(v0), Value::String(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 == v1));
                    }
                    _ => {}
                }
            }
            Inst::Ne(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 != v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 != v1));
                    }
                    (Value::String(v0), Value::String(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 != v1));
                    }
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Gt(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 > v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 > v1));
                    }
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Gte(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 >= v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 >= v1));
                    }
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Lt(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 < v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 < v1));
                    }
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Lte(r0, r1) => {
                let v0 = self.get_reg(r0);
                let v1 = self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 <= v1));
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.set_reg(Reg::ZF, &Value::Bool(v0 <= v1));
                    }
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Jump(address) => {
                self.set_pc(address);
            }
            Inst::JumpIf(address) => {
                let value = self.get_reg(Reg::ZF);
                match value {
                    Value::Bool(true) => {
                        self.set_pc(address);
                    }
                    Value::Bool(false) => {}
                    _ => panic!("Invalid comparison"),
                }
            }
            Inst::Hlt => {
                self.running = false;
            }
            Inst::Send(reg, reg1) => todo!(),
            Inst::Recv(reg) => todo!(),
            Inst::Push(reg) => {
                let value = self.get_reg(reg).clone();
                self.stack.push(value);
                self.set_reg(reg, &Value::Ref(0));
            }
            Inst::Pop(reg) => {
                let value = self.stack.pop().unwrap();
                self.set_reg(reg, &value);
            }
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
            heap: Vec::with_capacity(1000),
            mailbox: Mailbox::new(),
            lock: std::sync::Mutex::new(()),
            program: program,
            sender: sender,
            running: true,
        }
    }
}

fn sender(value: Value, to: Value) {
    // Implement the sender function
    println!("Sender function called, value: {:?}, to: {:?}", value, to);
}

fn main() {
    let pro = vec![
        Inst::Int(Reg::R0, 1),   // max
        Inst::Int(Reg::R1, 123), // sum
        Inst::List(Reg::R2, 10), // list
        Inst::SetC(Reg::R2, Reg::R0, Reg::R1),
        Inst::Hlt,
    ];
    let mut actor = ActorVm::new(pro, sender, 1000);
    while actor.running {
        actor.show_reg();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer);
        actor.tick();
    }
    actor.show_reg();
}
