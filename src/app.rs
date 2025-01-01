use std::borrow::Cow;

use gtk::Orientation;
use relm4::component::{AsyncComponent, AsyncComponentSender, AsyncComponentParts};
use relm4::once_cell::sync::OnceCell;

use gtk::prelude::{ApplicationExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt};

use crate::anchor::Anchor;
use crate::cal;
use crate::style::{self, StyleSettings};

pub static WM_CONFIG: OnceCell<WMConfig> = const { OnceCell::new() };

pub struct App;

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
                    add_css_class: "header",

                    #[name(month)]
                    gtk::Label {
                        add_css_class: "month",
                    }
                },

                #[name(grid)]
                gtk::Grid {
                    set_column_homogeneous: true,
                    set_row_homogeneous: true,
                }
            },
        }
    }

    fn init_loading_widgets(window: Self::Root) -> Option<relm4::loading_widgets::LoadingWidgets> {
        let config = WM_CONFIG.get().unwrap();

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

        let model = App;

        let widgets = view_output!();

        for (i, day) in cal::weekdays_with_first(&config.first).iter().enumerate() {
            let label = gtk::Label::new(Some(day));
            label.add_css_class("weekday");

            widgets.grid.attach(&label, i as i32, 0, 1, 1);
        }

        let (year, month, today) = cal::date();

        widgets.month.set_label(&cal::monthname(month));

        let day_for = cal::day_for(year, month, &config.first);

        const ROWS: i32 = 6;
        const COLUMNS: i32 = cal::WEEKDAYS.len() as i32;

        for column in 0..COLUMNS {
            for row in 0..ROWS {
                let day = day_for(column as _, row as _);
                let s = format!("{}", day.unsigned_abs());

                let label = gtk::Label::new(Some(&s));
                label.add_css_class("day");

                if day > 0 {
                    label.add_css_class("current");
                }

                if day == today as i8 {
                    label.add_css_class("today");
                }

                widgets.grid.attach(&label, column, row + 1, 1, 1);
            }
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update_cmd(&mut self, message: Self::CommandOutput, _: AsyncComponentSender<Self>, _: &Self::Root) {
        match message {
            CommandMessage::SetStyle(style) => relm4::set_global_css(&style),
            CommandMessage::Quit => relm4::main_application().quit(),
        }
    }
}
