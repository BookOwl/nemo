use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::cell::RefCell;
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
    EmptyBlock(String),
}

#[derive(Clone)]
pub enum Value<'a> {
    Number(f64),
    PrimFunc(Arc<Box<Fn(Vec<Value>) -> Value>>),
    UserFunc(Definition, &'a RefEnv<'a>),
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "Number({:?})", n),
            Value::PrimFunc(_) => write!(f, "PrimFunc {{...}}"),
            Value::UserFunc(ref def, _) => {
                write!(f, "UserFunc {}(", def.prototype.name);
                if def.prototype.args.len() >= 1 {
                    write!(f, "{}", def.prototype.args[0]);
                    write!(f, "{}", def.prototype.args.
                                                  iter().
                                                  skip(1).
                                                  fold(String::from(""), |res, ref arg| format!("{}, {}", res, arg)));
                }
                write!(f, ")")
            },
        }
    }
}
impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) =>  write!(f, "{}", n),
            Value::PrimFunc(_) => write!(f, "PrimFunc(...)"),
            Value::UserFunc(ref def, _) => {
                write!(f, "{}(", def.prototype.name);
                if def.prototype.args.len() >= 1 {
                    write!(f, "{}", def.prototype.args[0]);
                    write!(f, "{}", def.prototype.args.
                                                  iter().
                                                  skip(1).
                                                  fold(String::from(""), |res, ref arg| format!("{}, {}", res, arg)));
                }
                write!(f, ")")
            },
        }
    }
}

// The format and operations of the Enviroment are inspired by SICP's scheme interpreter.
// https://mitpress.mit.edu/sicp/full-text/book/book-Z-H-26.html
#[derive(Debug, Clone)]
pub struct Enviroment<'a> {
    current_frame: HashMap<String, Option<Value<'a>>>,
    prev: Option<&'a Enviroment<'a>>,
}
impl<'a> Enviroment<'a> {
    pub fn new() -> Enviroment<'a> {
        Enviroment {
            current_frame: HashMap::new(),
            prev: None,
        }
    }
    pub fn extend(bindings: Vec<(String, Value<'a>)>, prev: Option<&'a Enviroment<'a>>) -> Enviroment<'a> {
        let mut frame = HashMap::new();
        for (key, val) in bindings {
            frame.insert(key, Some(val));
        }
        Enviroment {
            current_frame: frame,
            prev: prev,
        }
    }
    pub fn lookup(&self, name: &str) -> Option<Option<Value<'a>>> {
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
    pub fn set(&mut self, name: String, val: Option<Value<'a>>) {
        self.current_frame.insert(name, val);
    }
}

type RefEnv<'a> = RefCell<Enviroment<'a>>;

pub fn define_function<'a>(def: Definition, env: &'a RefEnv<'a>) {
    let name = def.prototype.name.clone();
    let func = Value::UserFunc(def, env);
    env.borrow_mut().set(name, Some(func));
}

pub fn run<'a>(source: &'a str, env: &'a RefEnv) -> Result<(), Error<'a>> {
    let ast = parser::parse_Program(source).map_err(Error::ParseError)?;
    run_parsed_program(ast, &env)
}

pub fn initial_enviroment<'a>() -> Enviroment<'a> {
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
    Enviroment::extend(builtins, None)
}

pub fn eval<'a, 'b>(ast: &'a Expr, env: &'b RefEnv<'b>) -> Result<Value<'b>, Error<'b>> {
    match *ast {
        Expr::Number(n)            => Ok(Value::Number(n)),
        Expr::Binary(ref lhs, ref op, ref rhs) => {
            let l = eval(&*lhs, env)?;
            let r = eval(&*rhs, env)?;
            match *op {
                Op::Plus  => operations::plus(&l, &r),
                Op::Minus => operations::minus(&l, &r),
                Op::Times => operations::times(&l, &r),
                Op::Slash => operations::slash(&l, &r),
                _ => Err(Error::Unimplemented(format!("Operation {:?} is not implemented yet", op)))
            }
        },
        Expr::Name(ref name) => {
            let e = env.borrow();
            let val = e.lookup(&name);
            if let Some(Some(v)) = val {
                Ok(v)
            } else {
                Err(Error::UndefinedName(format!("{} is not defined", name)))
            }
        }
        Expr::Call(ref func, ref arg_exprs) => {
            let func = eval(func, env)?;
            let mut args = Vec::new();
            for arg in arg_exprs {
                args.push(eval(arg, env)?);
            }
            match func {
                Value::PrimFunc(f) => {
                    Ok(f(args))
                },

                _ => Err(Error::InvalidTypes(format!("{} is not a function!", func)))
            }
        },
        Expr::Assignment(ref name, ref val) => {
            let name = name.clone();
            let evaled_val = eval(val, env)?;
            env.borrow_mut().set(String::from(name), Some(evaled_val));
            Ok(Value::Number(0.0))
        },
        Expr::Block(ref expressions) => {
            let mut last = None;
            for expr in expressions {
                last = Some(eval(expr, env)?);
            };
            if last.is_none() {
                return Err(Error::EmptyBlock(s!("Empty blocks can not be evaluated.")))
            }
            Ok(last.unwrap())
        }
        ref x => Err(Error::Unimplemented(format!("{:?} is not implemented yet", x)))
    }
}

fn run_parsed_program<'a, 'b>(program: Vec<Definition>, env: &RefEnv) -> Result<(), Error<'a>> {
    unimplemented!()
}

mod operations {
    use super::*;
    pub fn plus<'a>(l: &Value, r: &Value) -> Result<Value<'a>, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 + n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for + {:?} and {:?}", l, r)))
        }
    }
    pub fn minus<'a>(l: &Value, r: &Value) -> Result<Value<'a>, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 - n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for - {:?} and {:?}", l, r)))
        }
    }
    pub fn times<'a>(l: &Value, r: &Value) -> Result<Value<'a>, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 * n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for * {:?} and {:?}", l, r)))
        }
    }
    pub fn slash<'a>(l: &Value, r: &Value) -> Result<Value<'a>, Error<'a>> {
        if let (&Value::Number(n1), &Value::Number(n2)) = (l, r) {
            Ok(Value::Number(n1 / n2))
        } else {
            Err(Error::InvalidTypes(format!("Invalid types for / {:?} and {:?}", l, r)))
        }
    }
}
