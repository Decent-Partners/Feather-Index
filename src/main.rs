use std::{
    path::PathBuf,
    process::exit,
    sync::{Arc, atomic::AtomicBool},
};

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use color_eyre::eyre::Result;
use futures::StreamExt;
use signal_hook::{consts::TERM_SIGNALS, flag};
use signal_hook_tokio::Signals;
use sled::Tree;
use subxt::{
    OnlineClient, PolkadotConfig, backend::rpc::RpcClient, ext::subxt_rpcs::LegacyRpcMethods,
};
use tokio::{join, spawn, sync::watch};
use tracing_log::{
    AsTrace,
    log::{error, info},
};

pub mod shared;
pub mod substrate;

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
    // https://docs.rs/signal-hook/0.3.17/signal_hook/#a-complex-signal-handling-with-a-background-thread
    // Make sure double CTRL+C and similar kills.
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now)).unwrap();
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will
        // first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&term_now)).unwrap();
    }
    // Create a watch channel to exit the program.
    let (exit_tx, exit_rx) = watch::channel(false);
    // Start indexer thread.
    let finalized = false;
    let queue_depth = 1;
    let substrate_index = spawn(substrate::substrate_index(
        trees.clone(),
        api.clone(),
        rpc.clone(),
        finalized,
        queue_depth,
        exit_rx.clone(),
    ));
    // Wait for signal.
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    signals.next().await;
    info!("Exiting.");
    let _ = exit_tx.send(true);
    // Wait to exit.
    let _result = join!(substrate_index);
    // Close db.
    // let _ = close_trees::<R>(trees);
    exit(0);
}
