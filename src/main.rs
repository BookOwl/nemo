extern crate nemo;
extern crate coroutine;
use coroutine::asymmetric::*;

use std::io::{stdin, stdout, Write};
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    expression_repl();
}

fn expression_repl() {
    let mut in_ = Coroutine::spawn(|mut h, _| {
        let env = Arc::new(RefCell::new(nemo::interpreter::initial_enviroment()));
        let mut out = Coroutine::spawn(|h, mut datum| {
            loop {
                println!("REPL got pushed {}", nemo::interpreter::box_from_usize(datum));
                datum = h.borrow_mut().yield_with(0);
            };
        });
        let stdin = stdin();
        let mut stdout = stdout();
        println!("><> nemo v0.0.1 <><");
        println!("Use Ctrl-C to exit.");
        loop {
            print!("> ");
            stdout.flush().unwrap();
            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();
            if let Ok(def) = nemo::parser::parse_Definition(&input) {
                nemo::interpreter::define_function(def, env.clone());
            } else {
                let expr = match nemo::parser::parse_Expr(&input) {
                    Ok(expr) => expr,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        continue;
                    }
                };
                match nemo::interpreter::eval(&expr, env.clone(), h.clone(), out.clone()) {
                    Ok(res) => println!("{}", res),
                    Err(e)  => println!("Error: {:?}", e),
                };
            }
        }
    });
    loop {
        in_.borrow_mut().yield_with(nemo::interpreter::box_to_usize(Box::new(nemo::interpreter::Value::Number(3.14))));
    }
}
