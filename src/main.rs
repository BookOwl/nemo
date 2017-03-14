extern crate nemo;

use std::io::{stdin, stdout, Write};

fn main() {
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
            if parsed.is_ok() {
                println!("{:?}", parsed.unwrap());
            } else {
                let parsed = nemo::parser::parse_Statement(&input);
                println!("{:?}", parsed.unwrap());
            }
        }
    }
}
