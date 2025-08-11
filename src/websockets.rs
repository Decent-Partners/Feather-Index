use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use sled::Tree;
use subxt::utils::AccountId32;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch::Receiver,
};
use tokio_tungstenite::tungstenite;
use tracing_log::log::{debug, error, info};
use zerocopy::{FromBytes, IntoBytes};

use crate::Trees;
use crate::shared::*;

pub fn process_msg_status(span_db: &Tree) -> ResponseMessage {
    let mut spans = vec![];
    for (key, value) in span_db.into_iter().flatten() {
        let span_value = SpanDbValue::read_from_bytes(&value).unwrap();
        let start: u32 = span_value.start.into();
        let end: u32 = u32::from_be_bytes(key.as_ref().try_into().unwrap());
        let span = Span { start, end };
        spans.push(span);
    }
    ResponseMessage::Status(spans)
}

pub fn process_msg_get_feathers(
    feathers_db: &Tree,
    block_number: u32,
    limit: u32,
    account_id: Option<AccountId32>,
    genre: Option<String>,
) -> ResponseMessage {
    let mut feathers = vec![];
    let key = FeatherDbKey {
        block_number: block_number.into(),
        index: 0.into(),
        account_id: [0; 32].into(),
    };
    debug!("search key: {:?}", key.as_bytes());
    let mut iter = feathers_db.range(key.as_bytes()..);

    while let Some(Ok((key, value))) = iter.next_back() {
        debug!("key: {:?}", key);
        if let Ok(key) = FeatherDbKey::read_from_bytes(&key) {
            if let Some(account_id) = account_id.clone() {
                if key.account_id != account_id.0 {
                    continue;
                }
            }

            let remark: String = value.to_vec().try_into().unwrap();

            if let Some(genre) = genre.clone() {
                let components: Vec<&str> = remark.split("::").collect();
                if components[1].to_string() != genre {
                    continue;
                }
            }
            feathers.push(Feather {
                block_number: key.block_number.into(),
                index: key.index.into(),
                account_id: subxt::utils::AccountId32(key.account_id),
                remark: remark,
            });

            let len: u32 = feathers.len().try_into().unwrap();

            if len == limit {
                break;
            }
        }
    }
    ResponseMessage::Feathers(feathers)
}

pub async fn process_msg(
    trees: &Trees,
    msg: RequestMessage,
) -> Result<ResponseMessage, IndexError> {
    debug!("{:?}", msg);
    Ok(match msg {
        RequestMessage::Status => process_msg_status(&trees.span),
        RequestMessage::GetFeathers {
            block_number,
            limit,
            account_id,
            genre,
        } => process_msg_get_feathers(&trees.feather, block_number, limit, account_id, genre),
        RequestMessage::SizeOnDisk => ResponseMessage::SizeOnDisk(trees.root.size_on_disk()?),
    })
}

async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    trees: Trees,
) -> Result<(), IndexError> {
    info!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await?;
    info!("WebSocket connection established: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    // Create the channel for the substrate thread to send event messages to this thread.

    loop {
        tokio::select! {
              Some(Ok(msg)) = ws_receiver.next() => {
                  debug!("{:?}", msg);
                  if msg.is_text() || msg.is_binary() {
                      match serde_json::from_str(msg.to_text()?) {
                          Ok(request_json) => {
                              let response_msg = process_msg(&trees, request_json).await?;
                              let response_json = serde_json::to_string(&response_msg).unwrap();
                              ws_sender.send(tungstenite::Message::Text(response_json.into())).await?;
                          },
                          Err(error) => error!("{}", error),
                      }
                  }
              },
        }
    }
}

pub async fn websockets_listen(trees: Trees, port: u16, mut exit_rx: Receiver<bool>) {
    let mut addr = "0.0.0.0:".to_string();
    addr.push_str(&port.to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    loop {
        tokio::select! {
            biased;

            _ = exit_rx.changed() => {
                break;
            }
            Ok((stream, addr)) = listener.accept() => {
                tokio::spawn(handle_connection(
                    stream,
                    addr,
                    trees.clone(),
                ));
            }
        }
    }
}
