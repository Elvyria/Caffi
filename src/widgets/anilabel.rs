use gtk::{glib::{self, Object}, prelude::{ButtonExt, WidgetExt, WidgetExtManual}};

mod imp {
    use std::cell::RefCell;

    use glib::Properties;
    use glib::subclass::types::ObjectSubclass;
    use glib::subclass::object::ObjectImpl;

    use gtk::prelude::ObjectExt;
    use gtk::subclass::button::ButtonImpl;
    use gtk::subclass::widget::WidgetImpl;
    use gtk::subclass::prelude::DerivedObjectProperties;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::AniLabel)]
    pub struct AniLabel {
        #[property(get, set)]
        animated_text: RefCell<String>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AniLabel {
        const NAME: &'static str = "AniLabel";
        type Type = super::AniLabel;
        type ParentType = gtk::Button;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AniLabel {}
    impl WidgetImpl for AniLabel {}
    impl ButtonImpl for AniLabel {}
}

glib::wrapper! {
    pub struct AniLabel(ObjectSubclass<imp::AniLabel>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for AniLabel {
    fn default() -> Self {
        let label: Self = Object::builder().build();

        label.connect_realize(|label| {
            label.add_tick_callback(|label, frame_clock| {
                if frame_clock.frame_counter() % 4 != 0 {
                    return glib::ControlFlow::Continue
                }

                let text = label.animated_text();
                let current = label.label().unwrap_or_default();
                let mut current_chars = current.chars();

                let mut mix = String::new();

                for char in text.chars() {
                    mix.push(char);

                    if Some(char) != current_chars.next() {
                        // TODO: check if compiler is dumb
                        mix.push_str(&current_chars.collect::<String>());
                        break
                    }
                }

                label.set_label(&mix);

                glib::ControlFlow::Continue
            });
        });

        label
    }
}
