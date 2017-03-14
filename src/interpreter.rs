use std::collections::HashMap;
use std::fmt;
use lalrpop_util;
use ast::*;
use parser;

macro_rules! s {
    ($e:expr) => (String::from($e));
}
macro_rules! prim {
    ($e:expr) => (Value::PrimFunc(Box::new($e)));
}

#[derive(Debug, Clone)]
pub enum Error<'a> {
    ParseError(lalrpop_util::ParseError<usize, (usize, &'a str), ()>),
    InvalidType(String),
}

pub enum Value {
    Number(f64),
    PrimFunc(Box<Fn(Vec<Value>) -> Value>),
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "Number({:?})", n),
            Value::PrimFunc(_) => write!(f, "PrimFunc(...)")
        }
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "{}", n),
            Value::PrimFunc(_) => write!(f, "PrimFunc(...)")
        }
    }
}

// The format and operations of the Enviroment are inspired by SICP's scheme interpreter.
// https://mitpress.mit.edu/sicp/full-text/book/book-Z-H-26.html
struct Enviroment<'a> {
    current_frame: HashMap<String, Option<Value>>,
    prev: Option<&'a Enviroment<'a>>,
}
impl<'a> Enviroment<'a> {
    fn new() -> Enviroment<'a> {
        Enviroment {
            current_frame: HashMap::new(),
            prev: None,
        }
    }
    fn extend(bindings: Vec<(String, Value)>, prev: Option<&'a Enviroment<'a>>) -> Enviroment {
        let mut frame = HashMap::new();
        for (key, val) in bindings {
            frame.insert(key, Some(val));
        }
        Enviroment {
            current_frame: frame,
            prev: prev,
        }
    }
    fn lookup(&self, name: String) -> Option<&Option<Value>> {
        let val = self.current_frame.get(&name);
        if val.is_some() {
            val
        } else {
            if let Some(prev) = self.prev {
                prev.lookup(name)
            } else {
                None
            }
        }
    }
    fn define(&mut self, name: String) {
        self.current_frame.insert(name, None);
    }
    fn set_value(&mut self, name: String, val: Option<Value>) -> Result<(), ()> {
        let old = self.current_frame.insert(name, val);
        if old.is_some() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub fn run(source: &str) -> Result<(), Error> {
    let ast = parser::parse_Program(source).map_err(Error::ParseError)?;
    let env = initial_enviroment();
    run_ast(ast, &env)
}

pub fn eval(source: &str) -> Result<Value, Error> {
    let ast = parser::parse_Expr(source).map_err(Error::ParseError)?;
    let env = initial_enviroment();
    eval_ast(&ast, &env)
}

fn initial_enviroment<'a>() -> Enviroment<'a> {
    let builtins = vec![
        ( s!("print"), prim!(|val| {println!("{:?}", val); Value::Number(0.0)}) ),
    ];
    Enviroment::extend(builtins, None)
}

fn eval_ast<'a>(ast: &Expr, env: &Enviroment) -> Result<Value, Error<'a>> {
    unimplemented!()
}

fn run_ast<'a, 'b>(defs: Vec<Definition>, env: &Enviroment) -> Result<(), Error<'a>> {
    unimplemented!()
}
