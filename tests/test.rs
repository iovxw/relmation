#![feature(conservative_impl_trait, fn_traits, unboxed_closures)]

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate relm_attributes;
#[macro_use]
extern crate relm_test;

extern crate relmation;

use relmation::*;

#[test]
fn it_works() {
    #[derive(SimpleMsg)]
    enum Msg {
        M(i32)
    }

    Animation::new(|p| Msg::M(p)).from(10).to(20);
}
