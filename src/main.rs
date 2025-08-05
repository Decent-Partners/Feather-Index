use std::process::exit;

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use color_eyre::eyre::Result;
use subxt::{
    OnlineClient, PolkadotConfig, backend::rpc::RpcClient, ext::subxt_rpcs::LegacyRpcMethods,
};
use tracing_log::{
    AsTrace,
    log::{error, info},
};

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
    /// URL of Substrate node to connect to
    #[arg(short, long)]
    pub url: Option<String>,
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
    // Determine url of Substrate node to connect to.
    let url = match args.url {
        Some(url) => url,
        None => "wss://kusama-rpc.polkadot.io:443".into(),
    };
    info!("Connecting to: {}", url);
    let rpc_client = match RpcClient::from_url(&url).await {
        Ok(rpc_client) => rpc_client,
        Err(err) => {
            error!("Failed to connect: {}", err);
            exit(1);
        }
    };
    let api = match OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client.clone()).await {
        Ok(api) => api,
        Err(err) => {
            error!("Failed to connect: {}", err);
            exit(1);
        }
    };
    let rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    Ok(())
}
