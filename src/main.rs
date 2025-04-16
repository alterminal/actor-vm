#[derive(Debug, Clone, Copy)]
enum Value {
    Ref(u64),
    Int(i64),
    Float(f64),
    // String(String),
    // Atom(String),
    // List(Vec<Value>),
    // Tuple(Vec<Value>),
    // Map(std::collections::HashMap<Value, Value>),
}

// impl Clone for Value {
//     fn clone(&self) -> Self {
//         match self {
//             Value::Ref(r) => Value::Ref(*r),
//             Value::Int(i) => Value::Int(*i),
//             Value::Float(f) => Value::Float(*f),
//             Value::String(s) => Value::String(s.clone()),
//             // Value::Atom(a) => Value::Atom(a.clone()),
//             // Value::List(l) => Value::List(l.clone()),
//             // Value::Tuple(t) => Value::Tuple(t.clone()),
//             // Value::Map(m) => Value::Map(m.clone()),
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
enum Register {
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
    Int(Register, i64),
    Float(Register, f64),
    Ref(Register, u64),
    String(Register, String),
    Atom(Register, String),
    List(Register),
    Tuple(Register),
    Map(Register),
    Move(Register, Register),
    Store(Register, u64),
    Load(u64, Register),
    Send(String, Value),
    Recv,
    Add(Register, Register, Register),
    Sub(Register, Register, Register),
    Mul(Register, Register, Register),
    Div(Register, Register, Register),
    Mod(Register, Register, Register),
    Jump(u64),
    JumpIf(u64),
    Eq(Register, Register),
    Ne(Register, Register),
    Gt(Register, Register),
    Lt(Register, Register),
    Gte(Register, Register),
    Lte(Register, Register),
    Push(Register),
    Pop(Register),
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

struct ActorVm {
    registers: [Value; 11],
    stack: Vec<Value>,
    heap: Vec<Value>,
    mailbox: Mailbox,
    lock: std::sync::Mutex<()>,
    program: Vec<Inst>,
    sender: fn(Value, Value),
}

impl ActorVm {
    fn get_tick(&mut self) -> bool {
        return false;
    }

    fn release(&mut self) {
        // Release the lock
        self.lock.lock().unwrap();
    }

    fn pc(&mut self) -> u64 {
        let pc = &self.registers[Register::PC as usize];
        match pc {
            Value::Ref(r) => *r,
            _ => panic!("PC is not a reference"),
        }
    }

    fn set_pc(&mut self, pc: u64) {
        self.registers[Register::PC as usize] = Value::Ref(pc);
    }

    fn get_reg(&mut self, reg: Register) -> &Value {
        &self.registers[reg as usize]
    }

    fn set_reg(&mut self, reg: Register, value: &Value) {
        self.registers[reg as usize] = *value;
    }

    fn tick(&mut self) {
        let pc = self.pc();
        let inst: &Inst = &self.program[pc as usize];
        self.registers[Register::PC as usize] = Value::Ref(pc + 1);
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
                self.set_reg(reg, &Value::Ref(0)); // Placeholder for string
            }
            Inst::Atom(reg, ref s) => {
                self.set_reg(reg, &Value::Ref(0)); // Placeholder for atom
            }
            Inst::Move(r1, r2) => {
                let value = *self.get_reg(r1); // Dereference and clone the value
                self.set_reg(r2, &value); // Pass the cloned value
            }
            Inst::Add(r0, r1, r2) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.registers[r2 as usize] = Value::Int(v0 + v1);
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.registers[r2 as usize] = Value::Float(v0 + v1);
                    }
                    _ => {}
                }
            }
            Inst::Sub(r0, r1, r2) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.registers[r2 as usize] = Value::Int(v0 - v1);
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.registers[r2 as usize] = Value::Float(v0 - v1);
                    }
                    _ => {}
                }
            }
            Inst::Mul(r0, r1, r2) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.registers[r2 as usize] = Value::Int(v0 * v1);
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.registers[r2 as usize] = Value::Float(v0 * v1);
                    }
                    _ => {}
                }
            }
            Inst::Div(r0, r1, r2) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.registers[r2 as usize] = Value::Int(v0 / v1);
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.registers[r2 as usize] = Value::Float(v0 / v1);
                    }
                    _ => {}
                }
            }
            Inst::Mod(r0, r1, r2) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        self.registers[r2 as usize] = Value::Int(v0 % v1);
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        self.registers[r2 as usize] = Value::Float(v0 % v1);
                    }
                    _ => {}
                }
            }
            Inst::Eq(r0, r1) => {
                let v0 = *self.get_reg(r0);
                let v1 = *self.get_reg(r1);
                match (v0, v1) {
                    (Value::Int(v0), Value::Int(v1)) => {
                        if (v0 == v1) {
                            self.registers[Register::ZF as usize] = Value::Int(1);
                        } else {
                            self.registers[Register::ZF as usize] = Value::Int(0);
                        }
                    }
                    (Value::Float(v0), Value::Float(v1)) => {
                        if (v0 == v1) {
                            self.registers[Register::ZF as usize] = Value::Int(1);
                        } else {
                            self.registers[Register::ZF as usize] = Value::Int(0);
                        }
                    }
                    _ => self.registers[Register::ZF as usize] = Value::Int(0),
                }
            }
            Inst::Jump(addr) => {
                self.registers[Register::PC as usize] = Value::Ref(addr);
            }
            Inst::JumpIf(addr) => {
                let zf = *self.get_reg(Register::ZF);
                match zf {
                    Value::Int(1) => {
                        self.registers[Register::PC as usize] = Value::Ref(addr);
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
        for (i, reg) in self.registers.iter().enumerate() {
            println!("Register {:?}: {:?}", i, reg);
        }
    }
    fn new(program: Vec<Inst>, sender: fn(Value, Value)) -> ActorVm {
        let a: Value = Value::Ref(0);
        let b: Value = Value::Ref(0);
        let c: Value = Value::Ref(0);
        let d: Value = Value::Ref(0);
        let e: Value = Value::Ref(0);
        let f: Value = Value::Ref(0);
        let g: Value = Value::Ref(0);
        let h: Value = Value::Ref(0);
        let i: Value = Value::Ref(0);
        let j: Value = Value::Ref(0);
        let k: Value = Value::Ref(0);
        let register = [a, b, c, d, e, f, g, h, i, j, k];
        ActorVm {
            registers: register,
            stack: Vec::new(),
            heap: Vec::new(),
            mailbox: Mailbox::new(),
            lock: std::sync::Mutex::new(()),
            program: program,
            sender: sender,
        }
    }
}

fn sender(_from: Value, _to: Value) {
    // Implement the sender function
    println!("Sender function called");
}

fn main() {
    let mut actor = ActorVm::new(
        vec![
            Inst::Int(Register::R1, 123),
            Inst::Move(Register::R1, Register::R0),
            Inst::Add(Register::R0, Register::R1, Register::R2),
            Inst::Hlt,
        ],
        sender,
    );
    actor.tick();
    actor.show_reg();
    actor.tick();
    actor.show_reg();
    actor.tick();
    actor.show_reg();
}
