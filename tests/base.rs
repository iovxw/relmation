#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

extern crate relmation;

use std::time::Duration;

use gtk::{ButtonExt, Inhibit, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget};
use relm_attributes::widget;

use relmation::*;

use self::Msg::*;

#[derive(Clone)]
pub struct Model {
    counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Start,
    Set(i32),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model { counter: 0 }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Set(n) => model.counter = n,
            Start => (),
            Quit => gtk::main_quit(),
        }
    }

    fn update_command(relm: &Relm<Msg>, event: Msg, model: &mut Model) {
        if let Start = event {
            Animation::new(|p| Set(p))
                .from(10)
                .to(20)
                .duration(Duration::from_secs(10))
                .start(relm);
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="start_button"]
                gtk::Button {
                    clicked => Start,
                    label: "Start",
                },
                #[name="label"]
                gtk::Label {
                    text: &model.counter.to_string(),
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use relm;
    use relm_test::click;

    use super::Win;

    #[test]
    fn animation() {
        let component = relm::init_test::<Win>(()).unwrap();
        let widgets = component.widget();

        click(&widgets.start_button);
        assert_text!(widgets.label, 10);
    }
}
