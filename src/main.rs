extern crate nemo;
#[macro_use]
extern crate clap;
extern crate bounded_spsc_queue as queue;
use std::io::{stdin, stdout, Write};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use clap::{Arg, App};

fn main() {
    let matches = App::new("nemo")
                          .version(crate_version!())
                          .author("Matthew S. <stanleybookowl@gmail.com>")
                          .about("The nemo interpreter")
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to interpret"))
                          .arg(Arg::with_name("REPL")
                               .short("r")
                               .long("repl")
                               .help("Starts the REPL")
                               .conflicts_with("INPUT"))
                          .get_matches();
    if matches.is_present("REPL") || matches.value_of("INPUT").is_none() {
        repl();
    } else {
        run_progam_in_file(matches.value_of("INPUT").unwrap());
    }
}

fn repl() {
    let env = nemo::interpreter::initial_enviroment();
    let stdin = stdin();
    let mut stdout = stdout();
    let (repl_producer, consumer) = queue::make(1);
    let (repl_producer, consumer) = (Arc::new(Mutex::new(repl_producer)), Arc::new(Mutex::new(consumer)));
    let (producer, repl_consumer) = queue::make(1);
    let (producer, repl_consumer) = (Arc::new(Mutex::new(producer)), Arc::new(Mutex::new(repl_consumer)));
    let p = repl_producer.clone();
    thread::spawn(move|| {
        loop {
            let lock = p.lock().unwrap();
            lock.push(nemo::interpreter::Value::FinishedPipe);
        }
    });
    let c = repl_consumer.clone();
    thread::spawn(move|| {
        loop {
            let lock = c.lock().unwrap();
            println!("REPL got pushed {}", lock.pop());
        }
    });
    println!("><> nemo v{} <><", crate_version!());
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
            match nemo::interpreter::eval(&expr, env.clone(), consumer.clone(), producer.clone()) {
                Ok(res) | Err(nemo::interpreter::Error::EarlyReturn(res)) => println!("{}", res),
                Err(e)  => println!("Error: {:?}", e),
            };
        }
    }
}

fn run_progam_in_file(path: &str) {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
}
