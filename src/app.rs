use std::borrow::Cow;

use gtk::gdk::ScrollUnit;
use gtk::graphene::Point;
use gtk::{Align, EventControllerScrollFlags, IconTheme, Orientation, Window};
use gtk::prelude::{ApplicationExt, GtkWindowExt, OrientableExt, WidgetExt};

use relm4::component::{AsyncComponent, AsyncComponentSender, AsyncComponentParts};
use relm4::once_cell::sync::OnceCell;
use relm4::RelmWidgetExt;

use jiff::civil::Date;
use jiff::{ToSpan, Zoned};

use smallvec::SmallVec;

use crate::event::Event;
use crate::{cal, event};
use crate::anchor::Anchor;
use crate::style::{self, StyleSettings};
use crate::widgets::anilabel::AniLabel;
use crate::widgets::monthgrid::MonthGrid;

pub static WM_CONFIG: OnceCell<WMConfig> = const { OnceCell::new() };

#[tracker::track]
pub struct App {
    date:  Date,
    today: Date,
    drag:  f32,

    #[no_eq]
    events: SmallVec<[Event; 10]>
}

impl App {
    // TODO: load user events
    // async fn refresh_events(&mut self) -> Result<(), std::io::Error> {
        // Ok(())
    // }

    fn load_icons(window: &Window) {
        gtk::gio::resources_register_include!("icons.gresource").unwrap();
        let theme = IconTheme::for_display(&window.display());
        theme.add_resource_path("/icons");
    }
}

pub struct Config {
    pub first: String,
    pub userstyle: Option<std::path::PathBuf>,

    #[cfg(feature = "Accent")]
    pub accent: bool,
}

pub struct WMConfig {
    pub anchors: Anchor,
    pub margins: Vec<i32>,
}

#[derive(Debug)]
pub enum ElementMessage {
    PrevMonth,
    NextMonth,
    Drag(f64),
}

#[derive(Debug)]
pub enum CommandMessage {
    SetStyle(Cow<'static, str>),
    Quit,
}

#[relm4::component(pub, async)]
impl AsyncComponent for App {
    type Init = Config;
    type Input = ElementMessage;
    type Output = ();
    type CommandOutput = CommandMessage;

    view! {
        gtk::Window {
            set_resizable: false,
            set_title:     Some(crate::APP_NAME),
            set_decorated: false,

            gtk::Box {
                add_css_class: "calendar",
                set_orientation: Orientation::Vertical,

                gtk::Box {
                    gtk::Box {
                        set_orientation: Orientation::Vertical,
                        AniLabel {
                            add_css_class: "month",
                            #[track = "self.changed(Self::date())"]
                            set_animated_text: cal::monthname(model.date.month() as _),
                        },

                        gtk::Label {
                            add_css_class: "year",
                            #[track = "self.changed(Self::date())"]
                            set_label: &model.date.year().to_string(),
                            set_halign: Align::Start,
                        },
                    },

                    gtk::Box {
                        set_halign: Align::End,
                        set_expand: true,

                        #[name = "left"]
                        gtk::Image::from_icon_name("left") {
                            add_css_class: "icon",
                        },
                        #[name = "right"]
                        gtk::Image::from_icon_name("right") {
                            add_css_class: "icon",
                        }
                    }
                },

                #[name = "monthgrid"]
                MonthGrid {
                    #[track = "self.changed(Self::drag())"]
                    set_translate: {
                        const MAX_DISTANCE: f32 = 0.68;
                        const REQUIRED_STRENGTH: f32 = 0.34;

                        let mut x = (model.drag.abs() - 1.0).powi(2);
                        x = 1.0 + x * (MAX_DISTANCE * x - REQUIRED_STRENGTH);

                        Point::new(x.copysign(model.drag), 0.0)
                    },
                    set_max_weekday_chars: 2,
                    set_first: config.first,
                    #[track = "self.changed(Self::date())"]
                    set_date: (model.date.year() as _, model.date.month() as _),
                    add_controller = gtk::EventControllerScroll {
                        set_flags: EventControllerScrollFlags::HORIZONTAL | EventControllerScrollFlags::KINETIC,
                        connect_scroll[sender] => move |e, x, _| {
                            if e.unit() == ScrollUnit::Wheel { return glib::Propagation::Proceed }

                            sender.input(ElementMessage::Drag(x));
                            glib::Propagation::Stop
                        },
                        connect_decelerate[sender] => move |e, x, _| {
                            if e.unit() == ScrollUnit::Wheel { return }

                            const PULL_STRENGTH: f64 = 400.0;

                            let message = match x {
                                x if x >  PULL_STRENGTH => ElementMessage::PrevMonth,
                                x if x < -PULL_STRENGTH => ElementMessage::NextMonth,
                                _ => ElementMessage::Drag(0.0),
                            };

                            sender.input(message);
                        }
                    },
                    #[track = "self.changed(Self::date())"]
                    set_event: event::today(),
                },
            },
        }
    }

    fn init_loading_widgets(window: Self::Root) -> Option<relm4::loading_widgets::LoadingWidgets> {
        let config = WM_CONFIG.get().unwrap();

        window.connect_application_notify(Self::load_icons);

        #[cfg(feature = "Wayland")]
        if crate::xdg::is_wayland() {
            window.connect_realize(move |w| Self::init_wayland(w, config.anchors, &config.margins, true));
        }

        #[cfg(feature = "X11")]
        if crate::xdg::is_x11() {
            window.connect_realize(move |w| Self::realize_x11(w, config.anchors, config.margins.clone()));
        }

        None
    }

    async fn init(config: Self::Init, window: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        if std::env::var("GTK_DEBUG").is_err() {
            glib::log_set_writer_func(|_, _| glib::LogWriterOutput::Handled);
        }

        sender.oneshot_command(async move {
            #[allow(unused_mut)]
            let mut settings = StyleSettings::default();

            #[cfg(feature = "Accent")]
            { settings.accent = config.accent; }

            let style = match config.userstyle {
                Some(p) => style::read(p).await,
                None    => {
                    let config_dir = crate::config_dir().await.unwrap();
                    style::find(config_dir, settings).await
                },
            };

            let style = match style {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    style::default(settings).await
                }
            };

            CommandMessage::SetStyle(style)
        });

        sender.oneshot_command(async move {
            use tokio::signal::*;

            let mut stream = unix::signal(unix::SignalKind::interrupt()).unwrap();
            stream.recv().await;

            CommandMessage::Quit
        });

        let model = App {
            date:  Zoned::now().date(),
            today: Zoned::now().date(),
            drag:  0.0,

            events: SmallVec::new_const(),

            tracker: 0,
        };

        let widgets = view_output!();
        let controller = gtk::GestureClick::builder().button(1).build();
        controller.connect_pressed({
            let left = widgets.left.clone();
            let sender = sender.clone();
            move |_, _, _, _| {
                left.add_css_class("pressed");
                sender.input(ElementMessage::PrevMonth)
            }
        });
        controller.connect_released({
            let left = widgets.left.clone();
            move |_, _, _, _| left.remove_css_class("pressed")
        });

        widgets.left.add_controller(controller);

        let controller = gtk::GestureClick::builder().button(1).build();
        controller.connect_pressed({
            let right = widgets.right.clone();
            let sender = sender.clone();
            move |_, _, _, _| {
                right.add_css_class("pressed");
                sender.input(ElementMessage::NextMonth);
            }
        });
        controller.connect_released({
            let right = widgets.right.clone();
            move |_, _, _, _| right.remove_css_class("pressed")
        });

        widgets.right.add_controller(controller);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _: AsyncComponentSender<Self>, _: &Self::Root) {
        use ElementMessage::*;
        self.reset();

        match message {
            Drag(x) => {
                let drag = self.drag + (0.02 * x).copysign(x) as f32;
                self.set_drag(drag.clamp(-3.5, 3.5));

                if x == 0.0 { self.set_drag(0.0) }
            },
            PrevMonth => {
                self.set_date(self.date - 1.month());
                self.set_drag(0.0);
            },
            NextMonth => {
                self.set_date(self.date + 1.month());
                self.set_drag(0.0);
            },
        }
    }

    async fn update_cmd(&mut self, message: Self::CommandOutput, _: AsyncComponentSender<Self>, _: &Self::Root) {
        match message {
            CommandMessage::SetStyle(style) => relm4::set_global_css(&style),
            CommandMessage::Quit => relm4::main_application().quit(),
        }
    }

}
