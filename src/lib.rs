extern crate num;
extern crate relm;
extern crate futures;
extern crate tokio_core;

use std::time::{Instant, Duration};
use std::rc::Rc;
use std::cell::RefCell;

use relm::Relm;
use futures::Stream;
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

#[derive(Clone)]
pub struct Animation<P, MSG> {
    callback: Rc<Fn(P) -> MSG>,
    from: P,
    to: P,
    delay: Duration,
    duration: Duration,
    frame: Duration,
    recur: Loop,
}

impl<P, MSG> Animation<P, MSG>
    where P: Number,
          MSG: Clone + relm::DisplayVariant + Send
{
    fn state(&self) -> State<P, MSG> {
        State {
            first: true,
            current: self.from,
            instant: Instant::now(),
            loop_counter: if let Loop::N(n) = self.recur { n } else { 0 },
            done: false,

            animation: self.clone(),
        }
    }
}

pub struct State<P, MSG> {
    first: bool,
    current: P,
    instant: Instant,
    loop_counter: usize,
    done: bool,

    animation: Animation<P, MSG>,
}

impl<P, MSG> State<P, MSG>
    where P: Number,
          MSG: Clone + relm::DisplayVariant + Send
{
    fn update(&mut self) -> P {
        assert!(!self.done, "Animation was finish");

        if self.first {
            self.instant = Instant::now(); // for delay
            self.first = false;
        }

        if self.current >= self.animation.to {
            if !self.animation.recur.is_infinite() {
                self.loop_counter -= 1;
                assert!(self.loop_counter >= 1);
            }
            self.instant -= self.animation.duration;
        }

        let p = to_millisecond(self.instant.elapsed()) as f64 /
                to_millisecond(self.animation.duration) as f64;
        self.current = self.animation.from + (self.animation.to - self.animation.from).mulf64(p);

        if self.current >= self.animation.to {
            self.current = self.animation.to;
            if self.loop_counter == 1 {
                self.done = true;
            }
        }

        self.current
    }
}

impl<P, MSG> Animation<P, MSG>
    where P: Number + 'static,
          MSG: Clone + relm::DisplayVariant + Send + 'static
{
    pub fn new<F>(callback: F) -> Animation<P, MSG>
        where F: Fn(P) -> MSG + 'static
    {
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

    pub fn from(mut self, from: P) -> Animation<P, MSG> {
        self.from = from;
        self
    }

    pub fn to(mut self, to: P) -> Animation<P, MSG> {
        self.to = to;
        self
    }

    pub fn recur<L: Into<Loop>>(mut self, recur: L) -> Animation<P, MSG> {
        self.recur = recur.into();
        self
    }

    pub fn delay(mut self, delay: Duration) -> Animation<P, MSG> {
        self.delay = delay;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Animation<P, MSG> {
        self.duration = duration;
        self
    }

    pub fn start(&self, relm: &Relm<MSG>) {
        let state = Rc::new(RefCell::new(self.state()));
        let stream = Interval::new_at(Instant::now() + self.delay, self.frame, relm.handle())
            .unwrap()
            .map_err(|e| panic!(e))
            .and_then(move |_| if state.borrow().done {
                          Err(()) // break loop
                      } else {
                          let p = state.borrow_mut().update();
                          Ok((state.borrow().animation.callback)(p))
                      });
        relm.connect_exec_ignore_err(stream, |x| x);
    }
}

fn to_millisecond(t: Duration) -> u64 {
    (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
}
