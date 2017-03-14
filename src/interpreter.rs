use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use lalrpop_util;
use ast::*;
use parser;

macro_rules! s {
    ($e:expr) => (String::from($e));
}
macro_rules! prim {
    ($e:expr) => (Value::PrimFunc(Arc::new(Box::new($e))));
}

#[derive(Debug, Clone)]
pub enum Error<'a> {
    ParseError(lalrpop_util::ParseError<usize, (usize, &'a str), ()>),
    InvalidTypes(String),
    Unimplemented(String),
    UndefinedName(String),
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    PrimFunc(Arc<Box<Fn(Vec<Value>) -> Value>>),
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
    fn lookup(&self, name: &str) -> Option<Option<Value>> {
        let val = self.current_frame.get(&String::from(name));
        if val.is_some() {
            val.cloned()
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
        ( s!("print"), prim!(|val: Vec<Value>| {
            println!("{:?}", val);
            Value::Number(0.0)}) ),
        ( s!("square"), prim!(|val: Vec<Value>| {
            if let Value::Number(n) = val[0] {
                Value::Number(n*n)
            } else {
                panic!("square was not passed a number!")
            }
        }))
    ];
    Enviroment::extend(builtins, None)
}

fn eval_ast<'a>(ast: &Expr, env: &Enviroment) -> Result<Value, Error<'a>> {
    match *ast {
        Expr::Number(n)            => Ok(Value::Number(n)),
        Expr::Binary(ref lhs, ref op, ref rhs) => {
            let (l, r) = (eval_ast(&*lhs, env)?, eval_ast(&*rhs, env)?);
            match *op {
                Op::Plus  => operations::plus(&l, &r),
                Op::Minus => operations::minus(&l, &r),
                Op::Times => operations::times(&l, &r),
                Op::Slash => operations::slash(&l, &r),
                _ => Err(Error::Unimplemented(format!("Operation {:?} is not implemented yet", op)))
            }
        },
        Expr::Name(ref name) => {
            let val = env.lookup(&name);
            if let Some(Some(v)) = val {
                Ok(v)
            } else {
                Err(Error::UndefinedName(format!("{} is not defined", name)))
            }
        }
        Expr::Call(ref func, ref arg_exprs) => {
            let func = eval_ast(func, env)?;
            let mut args = Vec::new();
            for arg in arg_exprs {
                args.push(eval_ast(&arg, env)?);
            }
            match func {
                Value::PrimFunc(f) => {
                    Ok(f(args))
                }
                _ => Err(Error::InvalidTypes(format!("{} is not a function!", func)))
            }
        }
        ref x => Err(Error::Unimplemented(format!("{:?} is not implemented yet", x)))
    }
}

fn run_ast<'a, 'b>(defs: Vec<Definition>, env: &Enviroment) -> Result<(), Error<'a>> {
    unimplemented!()
}

mod operations {
    use super::*;
    pub fn plus<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 + n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for + {:?} and {:?}", l, r)))
        }
    }
    pub fn minus<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 - n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for - {:?} and {:?}", l, r)))
        }
    }
    pub fn times<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 * n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for * {:?} and {:?}", l, r)))
        }
    }
    pub fn slash<'a>(l: &Value, r: &Value) -> Result<Value, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 / n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for / {:?} and {:?}", l, r)))
        }
    }
}
