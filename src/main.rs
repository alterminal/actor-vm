enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Atom(String),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Map(std::collections::HashMap<Value, Value>),
}

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
    StoreMap(Register, Register), // Store a map
    LoadMap(Register, Register),  // Load a map
    PutTuple(Register, Register), // Put a tuple
    GetTuple(Register, Register), // Get a tuple
    PutList(Register, Register),  // Put a list
    GetList(Register, Register),  // Get a list
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
    stack: Vec<Value>,
    heap: Vec<Value>,
    mailbox: Mailbox,
    lock: std::sync::Mutex<()>,
    program: Vec<Inst>,
}

impl ActorVm {
    fn post(&mut self, value: Value) {
        self.mailbox.post(value);
    }
    fn new(program: Vec<Inst>) -> ActorVm {
        ActorVm {
            stack: Vec::new(),
            heap: Vec::new(),
            mailbox: Mailbox::new(),
            lock: std::sync::Mutex::new(()),
            program: program,
        }
    }
}

fn main() {
    let actor = ActorVm::new(vec![]);
}
