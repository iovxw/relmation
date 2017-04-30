extern crate num;
extern crate relm;
extern crate futures;
extern crate tokio_core;

use std::time::{Instant, Duration};
use std::rc::Rc;
use std::cell::RefCell;

use relm::Relm;
use futures::{Future, Stream};
use tokio_core::reactor::Interval;

#[derive(Copy, Clone)]
pub enum Loop {
    Infinite,
    N(usize),
}

impl Loop {
    fn is_infinite(&self) -> bool {
        if let Loop::Infinite = *self {
            true
        } else {
            false
        }
    }
}

impl From<bool> for Loop {
    fn from(infinite: bool) -> Loop {
        if infinite { Loop::Infinite } else { Loop::N(1) }
    }
}

impl From<usize> for Loop {
    fn from(n: usize) -> Loop {
        Loop::N(n)
    }
}

pub trait Number: num::Zero + num::One +
    std::ops::Add<Output = Self> + std::ops::Sub<Output = Self> +
    Mulf64 +
    PartialOrd + Copy {}

impl<T> Number for T
    where T: num::Zero + num::One +
             std::ops::Add<Output = Self> + std::ops::Sub<Output = Self> +
             Mulf64 +
             PartialOrd + Copy {}

pub trait Mulf64 {
    fn mulf64(self, rhs: f64) -> Self;
}

macro_rules! impl_mulf64 {
    ( $t: ty ) => (
        impl Mulf64 for $t {
            fn mulf64(self, rhs: f64) -> $t {
                (self as f64 * rhs) as $t
            }
        }
    );
}

impl_mulf64!(isize);
impl_mulf64!(i8);
impl_mulf64!(i16);
impl_mulf64!(i32);
impl_mulf64!(i64);
impl_mulf64!(usize);
impl_mulf64!(u8);
impl_mulf64!(u16);
impl_mulf64!(u32);
impl_mulf64!(u64);
impl_mulf64!(f32);
impl_mulf64!(f64);

pub struct Animation<P, MSG, F>
    where P: Number,
          MSG: Clone + relm::DisplayVariant + Send,
          F: Fn(P) -> MSG + 'static
{
    callback: Rc<F>,
    from: P,
    to: P,
    delay: Duration,
    duration: Duration,
    frame: Duration,
    recur: Loop,
}

impl<P, MSG, F> Animation<P, MSG, F>
    where P: Number,
          MSG: Clone + relm::DisplayVariant + Send,
          F: Fn(P) -> MSG + 'static
{
    fn state(&self) -> State<P> {
        State {
            first: true,
            current: self.from,
            instant: Instant::now(),
            loop_counter: if let Loop::N(n) = self.recur { n } else { 0 },
            done: false,

            from: self.from,
            to: self.to,
            delay: self.delay,
            duration: self.duration,
            frame: self.frame,
            recur: self.recur,
        }
    }
}

pub struct State<P> {
    first: bool,
    current: P,
    instant: Instant,
    loop_counter: usize,
    done: bool,

    from: P,
    to: P,
    delay: Duration,
    duration: Duration,
    frame: Duration,
    recur: Loop,
}

impl<P> State<P>
    where P: Number
{
    fn update(&mut self) -> P {
        assert!(!self.done, "Animation was finish");

        if self.first {
            self.instant = Instant::now(); // for delay
            self.first = false;
        }

        if self.current >= self.to {
            if !self.recur.is_infinite() {
                self.loop_counter -= 1;
                assert!(self.loop_counter >= 1);
            }
            self.instant -= self.duration;
        }

        let p = to_millisecond(self.instant.elapsed()) as f64 /
                to_millisecond(self.duration) as f64;
        self.current = self.from + (self.to - self.from).mulf64(p);

        if self.current >= self.to {
            self.current = self.to;
            if self.loop_counter == 1 {
                self.done = true;
            }
        }

        self.current
    }
}

impl<P, MSG, F> Animation<P, MSG, F>
    where P: Number + 'static,
          MSG: Clone + relm::DisplayVariant + Send + 'static,
          F: Fn(P) -> MSG + 'static
{
    pub fn new(callback: F) -> Animation<P, MSG, F> {
        Animation {
            callback: Rc::new(callback),
            from: num::zero(),
            to: num::one(),
            delay: Duration::from_secs(0),
            duration: Duration::from_secs(1),
            frame: Duration::from_millis(16),
            recur: Loop::N(1),
        }
    }

    pub fn from(mut self, from: P) -> Animation<P, MSG, F> {
        self.from = from;
        self
    }

    pub fn to(mut self, to: P) -> Animation<P, MSG, F> {
        self.to = to;
        self
    }

    pub fn recur<L: Into<Loop>>(mut self, recur: L) -> Animation<P, MSG, F> {
        self.recur = recur.into();
        self
    }

    pub fn delay(mut self, delay: Duration) -> Animation<P, MSG, F> {
        self.delay = delay;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Animation<P, MSG, F> {
        self.duration = duration;
        self
    }

    pub fn start(&self, relm: &Relm<MSG>) {
        let state = Rc::new(RefCell::new(self.state()));
        let stream = Interval::new_at(Instant::now() + self.delay, self.frame, relm.handle())
            .unwrap()
            .map_err(|_| ())
            .and_then(move |_| {
                          if state.borrow().done {
                              return Err(()); // break loop
                          }
                          Ok(state.borrow_mut().update())
                      });
        let f = self.callback.clone();
        relm.connect_exec_ignore_err(stream, move |x| f(x));
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
