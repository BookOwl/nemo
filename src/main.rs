extern crate nemo;

use std::io::{stdin, stdout, Read, Write};

fn main() {
    let mut stdin = stdin();
    let mut stdout = stdout();
    loop {
        print!("> ");
        stdout.flush();
        let mut input = String::new();
        stdin.read_line(&mut input);
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
