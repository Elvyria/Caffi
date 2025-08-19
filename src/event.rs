use std::borrow::Cow;

use jiff::{civil::Date, Zoned};

pub fn today() -> Event {
    Event {
        active: true,
        class: "today".into(),
        start: Zoned::now().date(),
        end: None,
        repeat: Repeat::Never,
    }
}

pub struct Event {
    pub active: bool,
    pub class:  Cow<'static, str>,
    pub start:  Date,
    pub end:    Option<Date>,
    pub repeat: Repeat,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            active: true,
            class:  "event".into(),
            start:  Default::default(),
            end:    None,
            repeat: Repeat::Never
        }
    }
}

pub enum Repeat {
    Never,
    Weekly,
    Monthly,
    Yearly,
}

impl Default for Repeat {
    fn default() -> Self {
        Self::Never
    }
}
