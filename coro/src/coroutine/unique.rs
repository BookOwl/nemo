// The MIT License (MIT)

// Copyright (c) 2015 Rustcc developers

// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Basic single threaded Coroutine
//!
//! ```rust
//! use coroutine::{spawn, sched};
//!
//! let coro = spawn(|| {
//!     println!("Before yield");
//!
//!     // Yield back to its parent who resume this coroutine
//!     sched();
//!
//!     println!("I am back!");
//! });
//!
//! // Starts the Coroutine
//! coro.resume().ok().expect("Failed to resume");
//!
//! println!("Back to main");
//!
//! // Resume it
//! coro.resume().ok().expect("Failed to resume");
//!
//! println!("Coroutine finished");
//! ```
//!

/* Here is the coroutine(with scheduler) workflow:
 *
 *                               --------------------------------
 * --------------------------    |                              |
 * |                        |    v                              |
 * |                  ----------------                          |  III.Coroutine::yield_now()
 * |             ---> |   Scheduler  |  <-----                  |
 * |    parent   |    ----------------       |   parent         |
 * |             |           ^ parent        |                  |
 * |   --------------  --------------  --------------           |
 * |   |Coroutine(1)|  |Coroutine(2)|  |Coroutine(3)|  ----------
 * |   --------------  --------------  --------------
 * |         ^            |     ^
 * |         |            |     |  II.do_some_works
 * -----------            -------
 *   I.Handle.resume()
 *
 *
 *  First, all coroutines have a link to a parent coroutine, which was set when the coroutine resumed.
 *  In the scheduler/coroutine model, every worker coroutine has a parent pointer pointing to
 *  the scheduler coroutine(which is a raw thread).
 *  Scheduler resumes a proper coroutine and set the parent pointer, like procedure I does.
 *  When a coroutine is awaken, it does some work like procedure II does.
 *  When a coroutine yield(io, finished, paniced or sched), it resumes its parent's context,
 *  like procedure III does.
 *  Now the scheduler is awake again and it simply decides whether to put the coroutine to queue again or not,
 *  according to the coroutine's return status.
 *  And last, the scheduler continues the scheduling loop and selects a proper coroutine to wake up.
 */

use std::default::Default;
use thunk::Thunk;
use std::mem::transmute;
use std::rt::unwind::try;
use std::ops::Deref;
use std::ptr::Unique;
use std::fmt::{self, Debug};
use std::boxed;

use context::Context;
use stack::Stack;
use {State, Result, Error};
use options::Options;
use environment::Environment;

/// Handle of a Coroutine
pub struct Handle(Unique<Coroutine>);

impl Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            self.get_inner().name().fmt(f)
        }
    }
}

unsafe impl Send for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let p = Box::from_raw(*self.0);
            drop(p);
        }
    }
}

impl Handle {
    fn new(c: Coroutine) -> Handle {
        unsafe {
            Handle(Unique::new(boxed::into_raw(Box::new(c))))
        }
    }

    unsafe fn get_inner_mut(&self) -> &mut Coroutine {
        &mut **self.0
    }

    unsafe fn get_inner(&self) -> &Coroutine {
        & *self.0.get()
    }

    /// Resume the Coroutine
    pub fn resume(&self) -> Result {
        match self.state() {
            State::Finished => return Err(Error::Finished),
            State::Panicked => return Err(Error::Panicked),
            State::Normal => return Err(Error::Waiting),
            State::Running => return Ok(State::Running),
            _ => {}
        }

        let env = Environment::current();

        let from_coro_hdl = Coroutine::current();
        {
            let (from_coro, to_coro) = unsafe {
                (from_coro_hdl.get_inner_mut(), self.get_inner_mut())
            };

            // Save state
            to_coro.set_state(State::Running);
            from_coro.set_state(State::Normal);

            env.push(self);
            Context::swap(&mut from_coro.saved_context, &to_coro.saved_context);

            from_coro.set_state(State::Running);
        }

        match env.take_last_resume_result() {
            Some(err) => Err(Error::Panicking(err)),
            None => Ok(self.state()),
        }
    }

    /// Join this Coroutine.
    ///
    /// If the Coroutine panicked, this method will return an `Err` with panic message.
    ///
    /// ```ignore
    /// // Wait until the Coroutine exits
    /// spawn(|| {
    ///     println!("Before yield");
    ///     sched();
    ///     println!("Exiting");
    /// }).join().unwrap();
    /// ```
    #[inline]
    pub fn join(&self) -> Result {
        loop {
            match self.resume() {
                Ok(State::Finished) => break,
                Ok(..) => {},
                Err(Error::Finished) => break,
                Err(err) => return Err(err),
            }
        }
        Ok(State::Finished)
    }

    /// Get the state of the Coroutine
    #[inline]
    pub fn state(&self) -> State {
        unsafe {
            self.get_inner().state()
        }
    }

    /// Set the state of the Coroutine
    #[inline]
    fn set_state(&self, state: State) {
        unsafe { self.get_inner_mut().set_state(state) }
    }
}

impl Deref for Handle {
    type Target = Coroutine;

    #[inline]
    fn deref(&self) -> &Coroutine {
        unsafe { self.get_inner() }
    }
}

/// A coroutine is nothing more than a (register context, stack) pair.
#[allow(raw_pointer_derive)]
#[derive(Debug)]
pub struct Coroutine {
    /// The segment of stack on which the task is currently running or
    /// if the task is blocked, on which the task will resume
    /// execution.
    current_stack_segment: Option<Stack>,

    /// Always valid if the task is alive and not running.
    saved_context: Context,

    /// State
    state: State,

    /// Name
    name: Option<String>,
}

unsafe impl Send for Coroutine {}

/// Destroy coroutine and try to reuse std::stack segment.
impl Drop for Coroutine {
    fn drop(&mut self) {
        match self.current_stack_segment.take() {
            Some(stack) => {
                Environment::current().give_stack(stack);
            },
            None => {}
        }
    }
}

/// Initialization function for make context
extern "C" fn coroutine_initialize(_: usize, f: *mut ()) -> ! {
    let func: Box<Thunk> = unsafe { transmute(f) };

    let ret = unsafe { try(move|| func.invoke(())) };

    let env = Environment::current();

    let cur: &mut Coroutine = unsafe {
        env.running().get_inner_mut()
    };

    let state = match ret {
        Ok(..) => {
            env.set_resume_result(None);

            State::Finished
        }
        Err(err) => {
            if cfg!(feature = "enable-panic-message") {
                use std::io::stderr;
                use std::io::Write;
                let msg = match err.downcast_ref::<&'static str>() {
                    Some(s) => *s,
                    None => match err.downcast_ref::<String>() {
                        Some(s) => &s[..],
                        None => "Box<Any>",
                    }
                };

                let name = cur.name().unwrap_or("<unnamed>");

                let _ = writeln!(&mut stderr(), "Coroutine '{}' panicked at '{}'", name, msg);
            }

            env.set_resume_result(Some(err));

            State::Panicked
        }
    };

    loop {
        unsafe {
            Coroutine::try_switch(env, state);
        }
    }
}

impl Coroutine {

    #[doc(hidden)]
    pub unsafe fn empty(name: Option<String>, state: State) -> Handle {
        Handle::new(Coroutine {
            current_stack_segment: None,
            saved_context: Context::empty(),
            state: state,
            name: name,
        })
    }

    #[doc(hidden)]
    pub fn new(name: Option<String>, stack: Stack, ctx: Context, state: State) -> Handle {
        Handle::new(Coroutine {
            current_stack_segment: Some(stack),
            saved_context: ctx,
            state: state,
            name: name,
        })
    }

    /// Spawn a Coroutine with options
    pub fn spawn_opts<F>(f: F, opts: Options) -> Handle
        where F: FnOnce() + Send + 'static
    {

        let env = Environment::current();
        let mut stack = env.take_stack(opts.stack_size);

        let ctx = Context::new(coroutine_initialize, 0, f, &mut stack);

        Coroutine::new(opts.name, stack, ctx, State::Suspended)
    }

    /// Spawn a Coroutine with default options
    pub fn spawn<F>(f: F) -> Handle
        where F: FnOnce() + Send + 'static
    {
        Coroutine::spawn_opts(f, Default::default())
    }

    /// Yield the current running Coroutine to its parent
    #[inline]
    pub fn yield_now(state: State) {
        // Cannot yield with Running state
        assert!(state != State::Running);

        let env = Environment::current();
        unsafe {
            Coroutine::try_switch(env, state)
        }
    }

    #[inline(always)]
    unsafe fn try_switch(env: &mut Environment, state: State) {
        match (env.pop(), env.running()) {
            (Some(from_coro), to_coro) => {
                from_coro.set_state(state);
                Context::swap(&mut from_coro.get_inner_mut().saved_context,
                              &to_coro.saved_context);
            },
            // Environment root
            (None, _) => {}
        }
    }

    /// Yield the current running Coroutine with `Suspended` state
    #[inline]
    pub fn sched() {
        Coroutine::yield_now(State::Suspended)
    }

    /// Yield the current running Coroutine with `Blocked` state
    #[inline]
    pub fn block() {
        Coroutine::yield_now(State::Blocked)
    }

    /// Get a Handle to the current running Coroutine.
    ///
    /// It is unsafe because it is an undefined behavior if you resume a Coroutine
    /// in more than one native thread.
    #[inline]
    pub fn current() -> &'static Handle {
        Environment::current().running()
    }

    #[inline(always)]
    fn state(&self) -> State {
        self.state
    }

    #[inline(always)]
    fn set_state(&mut self, state: State) {
        self.state = state
    }

    /// Get the name of the Coroutine
    #[inline(always)]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| &**s)
    }

    /// Determines whether the current Coroutine is unwinding because of panic.
    #[inline(always)]
    pub fn panicking(&self) -> bool {
        self.state() == State::Panicked
    }

    /// Determines whether the Coroutine is finished
    #[inline(always)]
    pub fn finished(&self) -> bool {
        self.state() == State::Finished
    }
}
