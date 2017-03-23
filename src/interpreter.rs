use std::collections::HashMap;
use std::fmt;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::cmp::PartialEq;
use lalrpop_util;
use queue;
use ast::*;
use parser;

macro_rules! s {
    ($e:expr) => (String::from($e));
}
macro_rules! prim {
    ($e:expr) => (Value::PrimFunc(Arc::new(Box::new($e))));
}

pub fn box_to_usize(b: Box<Value>) -> usize {
    Box::into_raw(b) as usize
}

pub fn box_from_usize(p: usize) -> Box<Value> {
    unsafe {
        Box::from_raw(p as *mut Value)
    }
}

#[derive(Debug, Clone)]
pub enum Error<'a> {
    ParseError(lalrpop_util::ParseError<usize, (usize, &'a str), ()>),
    InvalidTypes(String),
    Unimplemented(String),
    UndefinedName(String),
    EmptyBlock(String),
    PushedToNone,
    // Not really an error, but treating early returns as one
    // is the easiest way to implement them.
    EarlyReturn(Value),
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    PrimFunc(Arc<Box<Fn(Vec<Value>) -> Value>>),
    UserFunc(Definition, ProtectedEnv),
    FinishedPipe,
    Bool(bool),
}
unsafe impl Send for Value{}
unsafe impl Sync for Value{}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "{}", n),
            Value::PrimFunc(_) => write!(f, "Primative {{...}}"),
            Value::UserFunc(ref def, _) => {
                write!(f, "function {}(", def.prototype.name);
                if def.prototype.args.len() >= 1 {
                    write!(f, "{}", def.prototype.args[0]);
                    write!(f, "{}", def.prototype.args.
                                                  iter().
                                                  skip(1).
                                                  fold(String::from(""), |res, ref arg| format!("{}, {}", res, arg)));
                }
                write!(f, ")")
            },
            Value::FinishedPipe => write!(f, "FinishedPipe"),
            Value::Bool(t) => write!(f, "{}", t),
        }
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "{}", n),
            Value::PrimFunc(_) => write!(f, "Primative {{...}}"),
            Value::UserFunc(ref def, _) => {
                write!(f, "function {}(", def.prototype.name);
                if def.prototype.args.len() >= 1 {
                    write!(f, "{}", def.prototype.args[0]);
                    write!(f, "{}", def.prototype.args.
                                                  iter().
                                                  skip(1).
                                                  fold(String::from(""), |res, ref arg| format!("{}, {}", res, arg)));
                }
                write!(f, ")")
            },
            Value::FinishedPipe => write!(f, "FinishedPipe"),
            Value::Bool(t) => write!(f, "{}", t),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Number(n1), &Value::Number(n2)) => n1 == n2,
            (&Value::FinishedPipe, &Value::FinishedPipe) => true,
            (&Value::Bool(b1), &Value::Bool(b2)) => b1 == b2,
            (x1, x2) => (x1 as *const Value as usize) == (x2 as *const Value as usize),
        }
    }
}

impl Value {
    pub fn truthy(&self) -> bool {
        // A Value is truthy if it is not Bool(false)
        self.ne(&Value::Bool(false))
    }
}

// The format and operations of the Enviroment are inspired by SICP's scheme interpreter.
// https://mitpress.mit.edu/sicp/full-text/book/book-Z-H-26.html
#[derive(Debug, Clone)]
pub struct Enviroment {
    current_frame: HashMap<String, Option<Value>>,
    prev: Box<Option<ProtectedEnv>>,
}
unsafe impl Send for Enviroment{}

impl Enviroment {
    pub fn new() -> Enviroment {
        Enviroment {
            current_frame: HashMap::new(),
            prev: Box::new(None),
        }
    }
    pub fn extend(bindings: Vec<(String, Value)>, prev: Option<ProtectedEnv>) -> Enviroment {
        let mut frame = HashMap::new();
        for (key, val) in bindings {
            frame.insert(key, Some(val));
        }
        Enviroment {
            current_frame: frame,
            prev: Box::new(prev),
        }
    }
    pub fn lookup(&self, name: &str) -> Option<Option<Value>> {
        let val = self.current_frame.get(&String::from(name));
        if val.is_some() {
            val.cloned()
        } else {
            if let Some(ref prev) = *self.prev {
                let lock = prev.lock().unwrap();
                return lock.borrow().lookup(name);
            } else {
                None
            }
        }
    }
    pub fn set(&mut self, name: String, val: Option<Value>) {
        self.current_frame.insert(name, val);
    }
}

type ProtectedEnv = Arc<Mutex<RefCell<Enviroment>>>;

pub fn define_function(def: Definition, env: ProtectedEnv) {
    let name = def.prototype.name.clone();
    let func = Value::UserFunc(def, env.clone());
    let lock = env.lock().unwrap();
    lock.borrow_mut().set(name, Some(func));
}

pub fn load_module_into_env<'a>(module: &'a str, env: ProtectedEnv) -> Result<(), Error<'a>> {
    let defs = parser::parse_Program(module).map_err(Error::ParseError)?;
    for def in defs {
        define_function(def, env.clone());
    }
    Ok(())
}

pub fn initial_enviroment() -> ProtectedEnv {
    let builtins = vec![
        ( s!("print"), prim!(|args: Vec<Value>| {
            for arg in args {
                print!("{} ", arg);
            }
            println!("");
            Value::Number(0.0)}) ),
        ( s!("square"), prim!(|val: Vec<Value>| {
            if let Value::Number(n) = val[0] {
                Value::Number(n*n)
            } else {
                panic!("square was not passed a number!")
            }
        }))
    ];
    let env = Arc::new(Mutex::new(RefCell::new(Enviroment::extend(builtins, None))));
    load_module_into_env(r#"
    range(n) => {
        x := 0;
        while x < n do {
            push x;
            x := x + 1;
        }
    }
    show_pipe() => {
        while true do {
            x := pull;
            if x = FinishedPipe then {
                return 0
            } else {
                print(x)
            };
        }
    }
    map(f) => {
        while true do {
            x := pull;
            if x = FinishedPipe then {
                return 0
            } else {
                push f(x)
            };
        }
    }
    filter(f) => {
        while true do {
            x := pull;
            if x = FinishedPipe then {
                return 0
            } else {
                if f(x) then push x else 0
            };
        }
    }
    "#, env.clone()).unwrap();
    env
}

pub fn eval<'a, 'b>(ast: &'a Expr, env: ProtectedEnv, this: Arc<Mutex<queue::Consumer<Value>>>, next: Arc<Mutex<queue::Producer<Value>>>) -> Result<Value, Error<'b>> {
    match *ast {
        Expr::Number(n) => Ok(Value::Number(n)),
        Expr::FinishedPipe => Ok(Value::FinishedPipe),
        Expr::Bool(b) => Ok(Value::Bool(b)),
        Expr::Lambda(ref args, ref body) => {
            let def = Definition::new(Prototype::new("lambda".to_owned(), args.clone()), body.clone());
            let func = Value::UserFunc(def, env.clone());
            Ok(func)
        }
        Expr::Push(ref val) => {
            let v = eval(val, env, this.clone(), next.clone())?;
            next.lock().unwrap().push(v);
            Ok(Value::Number(0.0))
        },
        Expr::Pull => {
            let val = this.lock().unwrap().pop();
            Ok(val)
        },
        Expr::Binary(ref lhs, Op::Pipe, ref rhs) => {
            let (send, recv) = queue::make(1);
            let (send, recv) = (Arc::new(Mutex::new(send)), Arc::new(Mutex::new(recv)));
            let l = lhs.clone();
            let e = env.clone();
            thread::spawn(move|| {
                eval(&l, e, this.clone(), send.clone()).unwrap();
                send.lock().unwrap().push(Value::FinishedPipe);
            });
            eval(rhs, env.clone(), recv, next)
        },
        Expr::Binary(ref lhs, ref op, ref rhs) => {
            let l = eval(&*lhs, env.clone(), this.clone(), next.clone())?;
            let r = eval(&*rhs, env.clone(), this.clone(), next.clone())?;
            match *op {
                Op::Plus    => operations::plus(&l, &r),
                Op::Minus   => operations::minus(&l, &r),
                Op::Times   => operations::times(&l, &r),
                Op::Slash   => operations::slash(&l, &r),
                Op::Percent => operations::percent(&l, &r),
                Op::Greater => operations::greater(&l, &r),
                Op::Lesser  => operations::lesser(&l, &r),
                Op::Equals  => operations::equals(&l, &r),
                Op::And     => operations::and(&l, &r),
                Op::Or      => operations::or(&l, &r),
                _ => Err(Error::Unimplemented(format!("Operation {:?} is not implemented yet", op)))
            }
        },
        Expr::Name(ref name) => {
            let e = env.lock().unwrap();
            let val = e.borrow().lookup(&name);
            if let Some(Some(v)) = val {
                Ok(v)
            } else {
                Err(Error::UndefinedName(format!("{} is not defined", name)))
            }
        }
        Expr::Call(ref func, ref arg_exprs) => {
            let func = eval(func, env.clone(), this.clone(), next.clone())?;
            let mut args = Vec::new();
            for arg in arg_exprs {
                args.push(eval(arg, env.clone(), this.clone(), next.clone())?);
            }
            match func {
                Value::PrimFunc(f) => {
                    Ok(f(args))
                },
                Value::UserFunc(ref def, ref body_env) => {
                    let mut new_bindings = vec![];
                    for i in 0..def.prototype.args.len() {
                        new_bindings.push((def.prototype.args[i].clone(), args[i].clone()))
                    }
                    let new_env = Arc::new(
                                  Mutex::new(
                                  RefCell::new(
                                      Enviroment::extend(new_bindings, Some(body_env.clone())
                                  ))));
                    match eval(&def.body, new_env, this.clone(), next) {
                        Err(Error::EarlyReturn(val)) => Ok(val),
                        r => r,
                    }
                }
                _ => Err(Error::InvalidTypes(format!("{} is not a function!", func)))
            }
        },
        Expr::Assignment(ref name, ref val) => {
            let name = name.clone();
            let evaled_val = eval(val, env.clone(), this.clone(), next.clone())?;
            let lock = env.lock().unwrap();
            lock.borrow_mut().set(String::from(name), Some(evaled_val));
            Ok(Value::Number(0.0))
        },
        Expr::Block(ref expressions) => {
            let mut last = None;
            for expr in expressions {
                last = Some(eval(expr, env.clone(), this.clone(), next.clone())?);
            };
            if last.is_none() {
                return Err(Error::EmptyBlock(s!("Empty blocks can not be evaluated.")))
            }
            Ok(last.unwrap())
        },
        Expr::If(ref cond, ref then, ref otherwise) => {
            if eval(cond, env.clone(), this.clone(), next.clone())?.truthy() {
                eval(then, env.clone(), this.clone(), next.clone())
            } else {
                eval(otherwise, env.clone(), this.clone(), next.clone())
            }
        },
        Expr::Return(ref val) => {
            Err(Error::EarlyReturn(eval(val, env.clone(), this.clone(), next.clone())?))
        },
        Expr::While(ref cond, ref body) => {
            while eval(cond, env.clone(), this.clone(), next.clone())?.truthy() {
                eval(body, env.clone(), this.clone(), next.clone())?;
            };
            Ok(Value::Number(0.0))
        }
        ref x => Err(Error::Unimplemented(format!("{:?} is not implemented yet", x)))
    }
}

fn run_parsed_program<'a>(program: Vec<Definition>, env: ProtectedEnv) -> Result<(), Error<'a>> {
    unimplemented!()
}

mod operations {
    use super::*;
    pub fn plus<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 + n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"+\": {:?} and {:?}", l, r)))
        }
    }
    pub fn minus<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 - n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"-\": {:?} and {:?}", l, r)))
        }
    }
    pub fn times<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 * n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"*\": {:?} and {:?}", l, r)))
        }
    }
    pub fn slash<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 / n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"/\": {:?} and {:?}", l, r)))
        }
    }
    pub fn percent<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 % n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"%\": {:?} and {:?}", l, r)))
        }
    }
    pub fn greater<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Bool(n1 > n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \">\": {:?} and {:?}", l, r)))
        }
    }
    pub fn lesser<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Bool(n1 < n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"<\": {:?} and {:?}", l, r)))
        }
    }
    pub fn equals<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        Ok(Value::Bool(l == r))
    }
    pub fn and<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Bool(n1), &Value::Bool(n2)) = (l, r) {
            Ok(Value::Bool(n1 && n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"and\": {:?} and {:?}", l, r)))
        }
    }
    pub fn or<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Bool(n1), &Value::Bool(n2)) = (l, r) {
            Ok(Value::Bool(n1 || n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for \"or\": {:?} and {:?}", l, r)))
        }
    }
}
