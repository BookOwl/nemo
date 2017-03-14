use std::collections::HashMap;
use lalrpop_util;
use ast::*;
use parser;

pub enum Error<'a> {
    ParseError(lalrpop_util::ParseError<usize, (usize, &'a str), ()>),
}

pub enum Value {
    Number(f64),
}

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
    fn extend(&'a self, bindings: Vec<(String, Value)>) -> Enviroment {
        let mut frame = HashMap::new();
        for (key, val) in bindings {
            frame.insert(key, Some(val));
        }
        Enviroment {
            current_frame: frame,
            prev: Some(self),
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
    unimplemented!()
}

fn eval_ast<'a>(ast: &Expr, env: &Enviroment) -> Result<Value, Error<'a>> {
    unimplemented!()
}

fn run_ast<'a, 'b>(defs: Vec<Definition>, env: &Enviroment) -> Result<(), Error<'a>> {
    unimplemented!()
}
