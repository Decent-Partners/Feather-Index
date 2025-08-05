use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use color_eyre::eyre::Result;
use tracing_log::AsTrace;

// https://github.com/rust-lang/cargo/blob/master/src/cargo/util/style.rs
pub fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, styles=get_styles())]
pub struct Args {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Check command line parameters.
    let args: Args = Args::parse();
    let log_level = args.verbose.log_level_filter().as_trace();
    tracing_subscriber::fmt().with_max_level(log_level).init();
    Ok(())
}
