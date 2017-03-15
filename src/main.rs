extern crate nemo;

use std::io::{stdin, stdout, Write};
use std::cell::RefCell;
use std::error::Error;

fn main() {
    expression_repl();
    //parse_repl()
}

fn expression_repl() {
    let stdin = stdin();
    let mut stdout = stdout();
    println!("><> nemo v0.0.1 <><");
    println!("Use Ctrl-C to exit.");
    let env = RefCell::new(nemo::interpreter::initial_enviroment());
    loop {
        print!("> ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let expr = match nemo::parser::parse_Expr(&input) {
            Ok(expr) => expr,
            Err(e) => {
                println!("Error: {:?}", e);
                continue;
            }
        };
        match nemo::interpreter::eval(&expr, &env) {
            Ok(res) => println!("{}", res),
            Err(e)  => println!("Error: {:?}", e),
        };
    }
}

fn parse_repl() {
    let stdin = stdin();
    let mut stdout = stdout();
    println!("><> nemo v0.0.1 <><");
    println!("Use Ctrl-C to exit.");
    loop {
        print!("> ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let parsed = nemo::parser::parse_Definition(&input);
        if parsed.is_ok() {
            println!("{:?}", parsed.unwrap());
        } else {
            let parsed = nemo::parser::parse_Expr(&input);
            println!("{:?}", parsed.unwrap());
        }
    }
}
