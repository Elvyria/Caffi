use std::path::PathBuf;

use error::{Error, ConfigError};
use anchor::Anchor;

static APP_NAME:   &str = "caffi";
static APP_ID:     &str = "elvy.caffi";
static APP_BINARY: &str = "caffi";

#[derive(argh::FromArgs)]
/// Calendar
struct Args {
    /// first day of the week: (sun)day, (mon)day, (tue)sday...
    #[argh(option, short = '1', default = "String::from(\"sunday\")")]
    first: String,

    /// screen anchor point: (t)op, (b)ottom, (l)eft, (r)ight
    #[argh(option, short = 'a', long = "anchor")]
    anchors: Vec<String>,

    /// margin distance for each anchor point
    #[argh(option, short = 'm', long = "margin")]
    margins: Vec<i32>,

    #[cfg(feature = "Accent")]
    /// inherit accent color from the system's settings
    #[argh(switch, short = 'C', long = "accent")]
    accent: bool,

    /// path to the userstyle
    #[argh(option, short = 'u', long = "userstyle")]
    userstyle: Option<PathBuf>,

    /// print version
    #[argh(switch, short = 'v', long = "version")]
    version: bool,
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    if args.version {
        print!("{}", env!("CARGO_PKG_NAME"));

        match option_env!("GIT_COMMIT") {
            Some(s) => println!(" {s}"),
            None    => println!(" {}", env!("CARGO_PKG_VERSION")),
        };

        return Ok(())
    }

    let mut anchors = Anchor::None;

    for a in args.anchors.iter().map(Anchor::try_from) {
        anchors |= a?;
    }

    warning(&args);

    let app = relm4::RelmApp::new(crate::APP_ID).with_args(vec![]);

    app::WM_CONFIG.get_or_init(|| app::WMConfig {
        anchors,
        margins: args.margins,
    });

    app.run_async::<app::App>(app::Config {
        first: args.first,
        userstyle: args.userstyle,

        #[cfg(feature = "Accent")]
        accent: args.accent,
    });

    Ok(())
}

#[allow(unused_variables)]
fn warning(args: &Args) {
    #[cfg(not(feature = "Wayland"))]
    if xdg::is_wayland() {
        warnln!("You are trying to use {APP_NAME} on Wayland, but '{}' feature wasn't included at compile time!", label::WAYLAND);
    }

    #[cfg(not(feature = "X11"))]
    if xdg::is_x11() {
        warnln!("You are trying to use {APP_NAME} on X Window System, but '{}' feature wasn't included at compile time!", label::X11);
    }

    #[cfg(not(feature = "Sass"))]
    if let Some(p) = &args.userstyle {
        let extension = p.extension().and_then(std::ffi::OsStr::to_str);
        if let Some("sass"|"scss") = extension {
            warnln!("You have specified *.{} file as userstyle, but '{}' feature wasn't included at compile time!", extension.unwrap(), label::SASS)
        }
    }
}

pub async fn config_dir() -> Result<PathBuf, ConfigError> {
    use tokio::fs;

    let mut dir = xdg::config_dir();
    dir.push(crate::APP_BINARY);

    let metadata = fs::metadata(&dir).await;

    match metadata {
        Err(_) => {
            fs::create_dir(&dir)
                .await
                .map_err(|e| ConfigError::Create { e, path: std::mem::take(&mut dir) })?;
        },
        Ok(metadata) => if !metadata.is_dir() {
            return Err(ConfigError::NotDirectory(dir))
        }
    }

    Ok(dir)
}

#[cfg(feature = "Accent")]
mod accent;
mod anchor;
mod app;
mod cal;
mod error;
mod event;
mod label;
mod proto;
mod style;
mod widgets;
mod xdg;
