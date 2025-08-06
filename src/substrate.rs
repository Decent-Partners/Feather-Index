use ahash::AHashMap;
use futures::future;
use num_format::{Locale, ToFormattedString};
use sled::Tree;
use subxt::ext::scale_value::{At, Composite, Primitive, Value, ValueDef};
use subxt::{
    OnlineClient, PolkadotConfig, blocks::Block, ext::subxt_rpcs::LegacyRpcMethods,
    metadata::Metadata,
};
use tokio::sync::RwLock;
use tokio::sync::watch;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tokio::time::{Duration, Instant};
use tracing_log::log::{debug, error, info};
use zerocopy::{FromBytes, IntoBytes};

use crate::Trees;

use crate::shared::*;

#[allow(clippy::type_complexity)]
pub struct Indexer {
    trees: Trees,
    api: Option<OnlineClient<PolkadotConfig>>,
    rpc: Option<LegacyRpcMethods<PolkadotConfig>>,
    metadata_map_lock: RwLock<AHashMap<u32, Metadata>>,
}

impl Indexer {
    fn new(
        trees: Trees,
        api: OnlineClient<PolkadotConfig>,
        rpc: LegacyRpcMethods<PolkadotConfig>,
    ) -> Self {
        Indexer {
            trees,
            api: Some(api),
            rpc: Some(rpc),
            metadata_map_lock: RwLock::new(AHashMap::new()),
        }
    }

    async fn index_head(
        &self,
        next: impl Future<
            Output = Option<
                Result<Block<PolkadotConfig, OnlineClient<PolkadotConfig>>, subxt::Error>,
            >,
        >,
    ) -> Result<(u32, u32, u32), IndexError> {
        let block = next.await.unwrap()?;
        self.index_block(block.number()).await
    }

    async fn index_block(&self, block_number: u32) -> Result<(u32, u32, u32), IndexError> {
        let mut key_count = 0;
        let api = self.api.as_ref().unwrap();
        let rpc = self.rpc.as_ref().unwrap();

        let block_hash = match rpc.chain_get_block_hash(Some(block_number.into())).await? {
            Some(block_hash) => block_hash,
            None => return Err(IndexError::BlockNotFound(block_number)),
        };

        let block = api.blocks().at(block_hash).await?;
        let extrinsics = block.extrinsics().await?;
        // Look for remarks.
        for xt in extrinsics.iter() {
            let address = xt.address_bytes().unwrap();
            info!("xt: {}, {}", xt.pallet_name()?, xt.variant_name()?);
            if (xt.pallet_name()? == "System") && (xt.variant_name()? == "remark") {
                let field_values = xt.field_values()?;

                if let Some(value) = field_values.at("remark") {
                    let mut remark = String::new();

                    let mut p = 0;

                    while let Some(char) = value.at(p) {
                        let c: u8 = char.as_u128().unwrap().try_into().unwrap();
                        remark.push(c.into());
                        p += 1;
                    }

                    info!("remark: {:#?}", remark);

                    let components: Vec<&str> = remark.split("::").collect();

                    if components[0] != "FEATHER" {
                        continue;
                    }

                    let genre = components[1];
                    let title = components[2];
                    let content = components[3];

                    info!("genre:  {:#?}", genre);
                    info!("title: {:#?}", title);
                    info!("content: {:#?}", content);
                }
            }
        }

        Ok((
            block_number,
            extrinsics.len().try_into().unwrap(),
            key_count,
        ))
    }
}

pub fn load_spans(span_db: &Tree) -> Result<Vec<Span>, IndexError> {
    let mut spans = vec![];
    for (key, value) in span_db.into_iter().flatten() {
        let span_value = SpanDbValue::read_from_bytes(&value).unwrap();
        let start: u32 = span_value.start.into();
        let end: u32 = u32::from_be_bytes(key.as_ref().try_into().unwrap());
        let span = Span { start, end };
        info!(
            "📚 Previous span of indexed blocks from #{} to #{}.",
            start.to_formatted_string(&Locale::en),
            end.to_formatted_string(&Locale::en)
        );
        spans.push(span);
    }
    Ok(spans)
}

pub fn check_span(
    span_db: &Tree,
    spans: &mut Vec<Span>,
    current_span: &mut Span,
) -> Result<(), IndexError> {
    while let Some(span) = spans.last() {
        // Have we indexed all the blocks after the span?
        if current_span.start > span.start && current_span.start - 1 <= span.end {
            let skipped = span.end - span.start + 1;
            info!(
                "📚 Skipping {} blocks from #{} to #{}",
                skipped.to_formatted_string(&Locale::en),
                span.start.to_formatted_string(&Locale::en),
                span.end.to_formatted_string(&Locale::en),
            );
            current_span.start = span.start;
            // Remove the span.
            span_db.remove(span.end.to_be_bytes())?;
            spans.pop();
        } else {
            break;
        }
    }
    Ok(())
}

pub fn check_next_batch_block(spans: &[Span], next_batch_block: &mut u32) {
    // Figure out the next block to index, skipping the next span if we have reached it.
    let mut i = spans.len();
    while i != 0 {
        i -= 1;
        if *next_batch_block >= spans[i].start && *next_batch_block <= spans[i].end {
            *next_batch_block = spans[i].start - 1;
        }
    }
}

pub async fn substrate_index(
    trees: Trees,
    api: OnlineClient<PolkadotConfig>,
    rpc: LegacyRpcMethods<PolkadotConfig>,
    finalized: bool,
    queue_depth: u32,
    mut exit_rx: watch::Receiver<bool>,
) -> Result<(), IndexError> {
    info!(
        "📇 Only index finalized blocks: {}",
        match finalized {
            false => "disabled",
            true => "enabled",
        },
    );

    let mut blocks_sub = if finalized {
        api.blocks().subscribe_finalized().await
    } else {
        api.blocks().subscribe_best().await
    }?;

    // Determine the correct block to start batch indexing.
    let mut next_batch_block: u32 = blocks_sub
        .next()
        .await
        .ok_or(IndexError::BlockNotFound(0))??
        .number();
    info!(
        "📚 Indexing backwards from #{}",
        next_batch_block.to_formatted_string(&Locale::en)
    );
    // Load already indexed spans from the db.
    let mut spans = load_spans(&trees.span)?;
    // If the first head block to be indexed will be touching the last span (the indexer was restarted), set the current span to the last span. Otherwise there will be no batch block indexed to connect the current span to the last span.
    let mut current_span = if let Some(span) = spans.last()
        && span.end == next_batch_block
    {
        let span = span.clone();
        let skipped = span.end - span.start + 1;
        info!(
            "📚 Skipping {} blocks from #{} to #{}",
            skipped.to_formatted_string(&Locale::en),
            span.start.to_formatted_string(&Locale::en),
            span.end.to_formatted_string(&Locale::en),
        );
        // Remove the span.
        trees.span.remove(span.end.to_be_bytes())?;
        spans.pop();
        next_batch_block = span.start - 1;
        span
    } else {
        Span {
            start: next_batch_block + 1,
            end: next_batch_block + 1,
        }
    };

    let indexer = Indexer::new(trees.clone(), api, rpc);

    let mut head_future = Box::pin(indexer.index_head(blocks_sub.next()));

    info!("📚 Queue depth: {}", queue_depth);
    let mut futures = Vec::with_capacity(queue_depth.try_into().unwrap());

    for _ in 0..queue_depth {
        check_next_batch_block(&spans, &mut next_batch_block);
        futures.push(Box::pin(indexer.index_block(next_batch_block)));
        debug!(
            "⬆️  Block #{} queued.",
            next_batch_block.to_formatted_string(&Locale::en)
        );
        next_batch_block -= 1;
    }

    let mut orphans: AHashMap<u32, ()> = AHashMap::new();

    let mut stats_block_count = 0;
    let mut stats_event_count = 0;
    let mut stats_key_count = 0;
    let mut stats_start_time = Instant::now();

    let interval_duration = Duration::from_millis(2000);
    let mut interval = time::interval_at(Instant::now() + interval_duration, interval_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut is_batching = true;

    loop {
        tokio::select! {
            biased;

            _ = exit_rx.changed() => {
                if current_span.start != current_span.end {
                    let value = SpanDbValue {
                        start: current_span.start.into(),
                    };
                    trees.span.insert(current_span.end.to_be_bytes(), value.as_bytes())?;
                    info!(
                        "📚 Recording current indexed span from #{} to #{}",
                        current_span.start.to_formatted_string(&Locale::en),
                        current_span.end.to_formatted_string(&Locale::en)
                    );
                }
                return Ok(());
            }
            result = &mut head_future => {
                match result {
                    Ok((block_number, event_count, key_count)) => {
                        trees.span.remove(current_span.end.to_be_bytes())?;
                        current_span.end = block_number;
                        let value = SpanDbValue {
                            start: current_span.start.into(),
                        };
                        trees.span.insert(current_span.end.to_be_bytes(), value.as_bytes())?;
                        info!(
                            "✨ #{}: {} extrinsics, {} keys",
                            block_number.to_formatted_string(&Locale::en),
                            event_count.to_formatted_string(&Locale::en),
                            key_count.to_formatted_string(&Locale::en),
                        );
                        drop(head_future);
                        head_future = Box::pin(indexer.index_head(blocks_sub.next()));
                    },
                    Err(error) => {
                        match error {
                            IndexError::BlockNotFound(block_number) => {
                                error!("✨ Block not found #{}", block_number.to_formatted_string(&Locale::en));
                            },
                            err => {
                                error!("✨ Indexing failed: {}", err);
                            },
                        }
                    },
                };
            }
            _ = interval.tick(), if is_batching => {
                let current_time = Instant::now();
                let duration = (current_time.duration_since(stats_start_time)).as_micros();
                if duration != 0 {
                    info!(
                        "📚 #{}: {} blocks/sec, {} extrinsics/sec, {} keys/sec",
                        current_span.start.to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_block_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_event_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_key_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                    );
                }
                stats_block_count = 0;
                stats_event_count = 0;
                stats_key_count = 0;
                stats_start_time = current_time;
            }
            (result, index, _) = future::select_all(&mut futures), if is_batching => {
                match result {
                    Ok((block_number, event_count, key_count)) => {
                        // Is the new block contiguous to the current span or an orphan?
                        if block_number == current_span.start - 1 {
                            current_span.start = block_number;
                            debug!("⬇️  Block #{} indexed.", block_number.to_formatted_string(&Locale::en));
                            check_span(&trees.span, &mut spans, &mut current_span)?;
                            // Check if any orphans are now contiguous.
                            while orphans.contains_key(&(current_span.start - 1)) {
                                current_span.start -= 1;
                                orphans.remove(&current_span.start);
                                debug!("➡️  Block #{} unorphaned.", current_span.start.to_formatted_string(&Locale::en));
                                check_span(&trees.span, &mut spans, &mut current_span)?;
                            }
                        }
                        else {
                            orphans.insert(block_number, ());
                            debug!("⬇️  Block #{} indexed and orphaned.", block_number.to_formatted_string(&Locale::en));
                        }
                        stats_block_count += 1;
                        stats_event_count += event_count;
                        stats_key_count += key_count;
                    },
                    Err(error) => {
                        match error {
                            IndexError::BlockNotFound(block_number) => {
                                error!("📚 Block not found #{}", block_number.to_formatted_string(&Locale::en));
                                is_batching = false;
                            },
                            _ => {
                                error!("📚 Batch indexing failed: {:?}", error);
                                is_batching = false;
                            },
                        }
                    }
                }
                check_next_batch_block(&spans, &mut next_batch_block);
                futures[index] = Box::pin(indexer.index_block(next_batch_block));
                debug!("⬆️  Block #{} queued.", next_batch_block.to_formatted_string(&Locale::en));
                next_batch_block -= 1;
            }
        }
    }
}
