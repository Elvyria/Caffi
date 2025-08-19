use gtk::glib::{self, Object};
use gtk::prelude::{GridExt, WidgetExt};

use super::monthgrid::MonthGrid;

pub const ROWS: i32 = 4;
pub const COLUMNS: i32 = crate::cal::MONTHS.len() as i32 / ROWS;

mod imp {
    use std::cell::Cell;

    use glib::Properties;
    use glib::subclass::types::ObjectSubclass;
    use glib::subclass::object::ObjectImpl;

    use gtk::prelude::ObjectExt;
    use gtk::subclass::grid::GridImpl;
    use gtk::subclass::widget::WidgetImpl;
    use gtk::subclass::orientable::OrientableImpl;
    use gtk::subclass::prelude::DerivedObjectProperties;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::YearGrid)]
    pub struct YearGrid {
        #[property(get, set)]
        active: Cell<f64>,
    }

    impl WidgetImpl for YearGrid {}
    impl OrientableImpl for YearGrid {}
    impl GridImpl for YearGrid {}

    #[glib::object_subclass]
    impl ObjectSubclass for YearGrid {
        const NAME: &'static str = "YearGrid";
        type Type = super::YearGrid;
        type ParentType = gtk::Grid;
    }

    #[glib::derived_properties]
    impl ObjectImpl for YearGrid {}
}

glib::wrapper! {
    pub struct YearGrid(ObjectSubclass<imp::YearGrid>)
        @extends gtk::Grid, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl YearGrid {
    pub fn from_date(year: u16) -> Self {
        let grid: Self = Object::builder().build();
        grid.add_css_class("year");

        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);

        grid.set_row_spacing(10);
        grid.set_column_spacing(10);

        for column in 0..COLUMNS {
            for row in 0..ROWS {
                let month = MonthGrid::from_date(year, (column * row  + column) as u8 + 1);
                grid.attach(&month, column, row, 1, 1);
            }
        }

        grid
    }
}
