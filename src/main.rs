use std::{path::PathBuf, process::exit};

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use color_eyre::eyre::Result;
use sled::Tree;
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
    /// Database path
    #[arg(short, long)]
    pub db_path: Option<String>,
    /// URL of Substrate node to connect to
    #[arg(short, long)]
    pub url: Option<String>,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

/// Database trees for the indexer
#[derive(Clone)]
pub struct Trees {
    pub root: sled::Db,
    pub span: Tree,
}

pub fn open_trees(db_config: sled::Config) -> Result<Trees, sled::Error> {
    let db = db_config.open()?;
    let trees = Trees {
        root: db.clone(),
        span: db.open_tree(b"span")?,
    };
    Ok(trees)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Check command line parameters.
    let args: Args = Args::parse();
    let log_level = args.verbose.log_level_filter().as_trace();
    tracing_subscriber::fmt().with_max_level(log_level).init();
    // Open database.
    let db_path = match args.db_path {
        Some(db_path) => PathBuf::from(db_path),
        None => match home::home_dir() {
            Some(mut db_path) => {
                db_path.push(".local/share/feather-index/");
                db_path.push("db");
                db_path
            }
            None => {
                error!("No home directory.");
                exit(1);
            }
        },
    };
    info!("Database path: {}", db_path.display());
    let db_config = sled::Config::new().path(db_path);
    let trees = match open_trees(db_config) {
        Ok(trees) => trees,
        Err(_) => {
            error!("Failed to open database.");
            exit(1);
        }
    };
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
