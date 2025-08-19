use glib::object::Cast;
use gtk::graphene::Point;
use gtk::glib::{self, Object};
use gtk::prelude::{GridExt,WidgetExt, WidgetExtManual};

use jiff::ToSpan;
use jiff::civil::Date;

use crate::cal::{self, CalendarDay};

pub const ROWS: u8 = 6;
pub const COLUMNS: u8 = crate::cal::WEEKDAYS.len() as u8;

mod imp {
    use std::cell::{Cell};

    use glib::Properties;
    use glib::subclass::types::ObjectSubclass;
    use glib::subclass::object::ObjectImpl;

    use gtk::graphene::Point;
    use gtk::prelude::{ObjectExt, SnapshotExt};
    use gtk::subclass::grid::GridImpl;
    use gtk::subclass::widget::{WidgetImpl, WidgetImplExt};
    use gtk::subclass::orientable::OrientableImpl;
    use gtk::subclass::prelude::DerivedObjectProperties;
    use gtk::Snapshot;

    use super::{COLUMNS};

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::MonthGrid)]
    pub struct MonthGrid {
        #[property(get, set)]
        first: std::cell::RefCell<String>,

        #[property(get, set)]
        year: Cell<u32>,

        #[property(get, set)]
        month: Cell<u8>,

        #[property(get, set)]
        max_weekday_chars: Cell<u8>,

        #[property(get, set)]
        translate: Cell<Point>,

        #[property(get, set)]
        month_range: Cell<Range>
    }

    impl WidgetImpl for MonthGrid {
        fn snapshot(&self, snapshot: &Snapshot) {
            // snapshot.copmute_point
            snapshot.translate(&self.translate.get());
            self.parent_snapshot(snapshot);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthGrid {
        const NAME: &'static str = "MonthGrid";
        type Type = super::MonthGrid;
        type ParentType = gtk::Grid;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthGrid {}
    impl OrientableImpl for MonthGrid {}
    impl GridImpl for MonthGrid {}

    // TODO: rename
    #[derive(Copy, Clone, Default, Debug, PartialEq)]
    pub struct GridCell { pub column: u8, pub row: u8 }

    impl GridCell {
        pub fn next(&self) -> GridCell {
            let column = (self.column + 1) % COLUMNS;
            let row = self.row + (self.column + 1) / COLUMNS;

            GridCell { column, row }
        }
    }

    impl From<u32> for GridCell {
        fn from(n: u32) -> Self {
            GridCell { column: (n >> 8) as u8, row: n as u8 }
        }
    }

    // impl<'a> From<&'a Pos> for u32 {
        // fn from(limits: &'a Pos) -> Self {
            // (limits.column as u32) << 8 | limits.row as u32
        // }
    // }

    impl From<GridCell> for u32 {
        fn from(pos: GridCell) -> Self {
            (pos.column as u32) << 8 | pos.row as u32
        }
    }

    #[derive(glib::ValueDelegate, Copy, Clone, Default)]
    #[value_delegate(from = u32)]
    pub struct Range {
        pub start: GridCell,
        pub end:   GridCell,
    }

    impl From<u32> for Range {
        fn from(n: u32) -> Self {
            Range { start: GridCell::from(n >> 16), end: GridCell::from(n) }
        }
    }

    impl<'a> From<&'a Range> for u32 {
        fn from(range: &'a Range) -> Self {
            u32::from(range.start) << 16 | u32::from(range.end)
        }
    }

    impl From<Range> for u32 {
        fn from(range: Range) -> Self {
            u32::from(range.start) << 16 | u32::from(range.end)
        }
    }
}

glib::wrapper! {
    pub struct MonthGrid(ObjectSubclass<imp::MonthGrid>)
        @extends gtk::Grid, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for MonthGrid {
    fn default() -> Self {
        let grid: Self = Object::builder().build();
        grid.add_css_class("month");
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        grid.connect_translate_notify(|grid| grid.queue_draw());
        grid.connect_realize(|grid| {
            grid.add_tick_callback(|grid, _| {
                const CLASS: &str = "visible";
                let mut next = grid.first_child();

                while let Some(ref child) = next {
                    if child.has_css_class(CLASS) {
                        next = child.next_sibling();
                        continue;
                    }

                    child.remove_css_class("invisible");
                    child.add_css_class(CLASS);
                    break
                }

                glib::ControlFlow::Continue
            });
        });

        grid
    }
}

impl MonthGrid {
    #[inline]
    fn date(&self) -> Date {
        Date::new(self.year() as _, self.month() as _, 1).unwrap()
    }

    #[inline]
    fn shows_date(current: Date, date: Date) -> bool {
        date.year() == current.year() || date.year() == (current + 1.month()).year() || date.year() == (current - 1.month()).year()
    }

    pub fn child_by_date(&self, date: Date) -> Option<gtk::Widget> {
        let current = self.date();

        if !MonthGrid::shows_date(current, date) {
            return None
        }

        let range = self.month_range();
        let mut column = -1;
        let mut row = -1;

        if date.month() == current.month() {
            let diff = ((range.start.row - 1) * COLUMNS + range.start.column + date.day() as u8 - 1) as i8;

            row = diff / COLUMNS as i8;
            column = diff % COLUMNS as i8;
        }

        // if date < current.first_of_month() {
            // let days = cal::days_in_month(date.year() as _, date.month() as _);

            // let zero = days - (COLUMNS * (range.start.row - 1)) - range.start.column;
            // let diff = date.day() - zero as i8;

            // row = diff / COLUMNS as i8;
            // column = diff % COLUMNS as i8;
        // }

        if date > current.last_of_month() {
        }

        self.child_at(column as _, (row + 1) as _)
    }

    pub fn set_event(&self, event: crate::event::Event) {
        // TODO: return result
        let Some(child) = self.child_by_date(event.start) else { return };
        let label = child.downcast::<gtk::Label>().unwrap();
        label.add_css_class(&event.class);
    }

    pub fn set_date(&self, year: u16, month: u8) {
        self.set_year(year as u32);
        self.set_month(month);
        self.set_translate(Point::zero());
        self.clear();
        self.fill();
    }

    pub fn from_date(year: u16, month: u8) -> Self {
        let grid: Self = Object::builder().build();
        grid.add_css_class("month");
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);

        grid.set_year(year as u32);
        grid.set_month(month);

        grid.fill();

        grid
    }

    fn clear(&self) {
        while let Some(child) = self.first_child() {
            self.remove(&child);
        }
    }

    fn fill(&self) {
        let first: &str = &self.first();
        let max = self.max_weekday_chars() as usize;

        for (i, day) in cal::weekdays_with_first(first).iter().enumerate() {
            // `set_max_width_chars` doesn't work, like pretty much everything else in GTK, whatever
            let day: String = day.chars().take(max).collect();
            let label = gtk::Label::new(Some(&day));
            label.add_css_class("weekday");

            self.attach(&label, i as i32, 0, 1, 1);
        }

        let day_for = cal::day_for(self.year() as _, self.month(), first);
        let first: u8 = cal::first_day(first);

        let (mut start, mut end) = (None, None);

        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let day = day_for(column, row);

                let label = gtk::Label::new(None);
                label.add_css_class("day");
                label.add_css_class("invisible");

                match day {
                    CalendarDay::Previous(_) => label.add_css_class("previous"),
                    CalendarDay::Current(_) => {
                        label.add_css_class("current");
                        if start.is_none() { start = Some(imp::GridCell { column, row: row + 1 }) }
                    },
                    CalendarDay::Next(_) => {
                        label.add_css_class("next");
                        if end.is_none() { end = Some(imp::GridCell { column, row: row + 1 }) }
                    },
                }

                let day: u8 = day.into();
                label.set_label(&day.to_string());

                if (column + first) % 6 == 0 || (column + first) % 7 == 0 {
                    label.add_css_class("weekend");
                }

                self.attach(&label, column as i32, row as i32 + 1, 1, 1);
            }
        }

        #[allow(clippy::unnecessary_unwrap)]
        if start.is_some() && end.is_some() {
            self.set_month_range(imp::Range { start: start.unwrap(), end: end.unwrap() });
        }
    }
}
