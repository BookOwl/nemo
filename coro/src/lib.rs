// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// #![license = "MIT/ASL2"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico")]

#![feature(fnbox, asm, test, unboxed_closures)]

#[macro_use] extern crate log;
extern crate libc;
extern crate test;
extern crate context;

use std::any::Any;
use std::error;
use std::fmt::{self, Display};
use std::panic;
use std::thread;

pub use options::Options;

pub mod asymmetric;
pub mod options;

/// Return type of resuming. Ok if resume successfully with the current state,
/// Err if resume failed with `Error`.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Resume Error
pub enum Error {
    /// Coroutine is panicked
    Panicked,

    /// Coroutine is panicking, carry with the parameter of `panic!()`
    Panicking(Box<Any + Send>),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Panicked => write!(f, "Panicked"),
            &Error::Panicking(ref err) => {
                let msg = match err.downcast_ref::<&'static str>() {
                    Some(s) => *s,
                    None => match err.downcast_ref::<String>() {
                        Some(s) => &s[..],
                        None => "Box<Any>",
                    }
                };
                write!(f, "Panicking({})", msg)
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Panicked => "Panicked",
            &Error::Panicking(..) => "Panicking(..)",
        }
    }
}

unsafe fn try<R, F: FnOnce() -> R>(f: F) -> thread::Result<R> {
    let mut f = Some(f);
    let f = &mut f as *mut Option<F> as usize;

    panic::catch_unwind(move || (*(f as *mut Option<F>)).take().unwrap()())
}
