# Feather Index

Indexes [Feathers](https://github.com/Decent-Partners/Feather-Protocol) published via system.remark or system.remarkWithEvent on Kusama or similar.

## Install

```
cargo build --release
```


## Run

```
/target/release/feather-index --help
```

```
Usage: feather-index [OPTIONS]

Options:
  -d, --db-path <DB_PATH>          Database path
  -u, --url <URL>                  URL of Substrate node to connect to
      --queue-depth <QUEUE_DEPTH>  Maximum number of concurrent requests to the chain [default: 1]
  -b, --best                       Load feathers from blocks before they are finalized
  -p, --port <PORT>                Port to open for WebSocket queries [default: 8172]
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -h, --help                       Print help
  -V, --version                    Print version
```

```
/target/release/feather-index
```

```
2025-08-11T07:28:09.338175Z  INFO feather_index: Database path: /home/jbrown/.local/share/feather-index/db    
2025-08-11T07:28:09.355336Z  INFO feather_index: Connecting to: wss://kusama-rpc.polkadot.io:443    
2025-08-11T07:28:12.374777Z  INFO feather_index::substrate: 📇 Load feathers before finalization: disabled    
2025-08-11T07:28:12.374846Z  INFO feather_index::websockets: Listening on: 0.0.0.0:8172    
2025-08-11T07:28:13.279000Z  INFO feather_index::substrate: 📚 Indexing backwards from #29,610,910    
2025-08-11T07:28:13.279048Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,531,039 to #29,532,459.    
2025-08-11T07:28:13.279053Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,541,562 to #29,541,569.    
2025-08-11T07:28:13.279056Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,541,623 to #29,541,641.    
2025-08-11T07:28:13.279060Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,543,202 to #29,544,860.    
2025-08-11T07:28:13.279063Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,545,218 to #29,545,243.    
2025-08-11T07:28:13.279066Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,545,572 to #29,545,608.    
2025-08-11T07:28:13.279068Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,552,162 to #29,554,030.    
2025-08-11T07:28:13.279071Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,554,200 to #29,554,813.    
2025-08-11T07:28:13.279073Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,554,862 to #29,554,881.    
2025-08-11T07:28:13.279079Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,568,542 to #29,568,892.    
2025-08-11T07:28:13.279082Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,570,066 to #29,583,952.    
2025-08-11T07:28:13.279085Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,596,879 to #29,598,618.    
2025-08-11T07:28:13.279087Z  INFO feather_index::substrate: 📚 Previous span of indexed blocks from #29,607,406 to #29,610,838.    
2025-08-11T07:28:13.279092Z  INFO feather_index::substrate: 📚 Queue depth: 1    
2025-08-11T07:28:15.281048Z  INFO feather_index::substrate: 📚 #29,610,909: 0 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:17.281352Z  INFO feather_index::substrate: 📚 #29,610,907: 0 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:18.167960Z  INFO feather_index::substrate: ✨ #29,610,911: 0 feathers    
2025-08-11T07:28:19.281346Z  INFO feather_index::substrate: 📚 #29,610,905: 0 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:21.280182Z  INFO feather_index::substrate: 📚 #29,610,903: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:23.280783Z  INFO feather_index::substrate: 📚 #29,610,901: 0 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:25.281376Z  INFO feather_index::substrate: 📚 #29,610,898: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:26.585753Z  INFO feather_index::substrate: ✨ #29,610,912: 0 feathers    
2025-08-11T07:28:27.281117Z  INFO feather_index::substrate: 📚 #29,610,896: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:29.280886Z  INFO feather_index::substrate: 📚 #29,610,894: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:31.053633Z  INFO feather_index::substrate: ✨ #29,610,913: 0 feathers    
2025-08-11T07:28:31.280608Z  INFO feather_index::substrate: 📚 #29,610,892: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:33.280975Z  INFO feather_index::substrate: 📚 #29,610,889: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:35.281238Z  INFO feather_index::substrate: 📚 #29,610,887: 0 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:37.280256Z  INFO feather_index::substrate: 📚 #29,610,885: 1 blocks/sec, 0 feathers/sec    
2025-08-11T07:28:39.281113Z  INFO feather_index::substrate: 📚 #29,610,882: 1 blocks/sec, 0 feathers/sec
```

## Query

```
cargo install websocat
websocat ws://0.0.0.0:8172
```

Query:
```
{"type": "GetFeathers", "block_number": 0, "limit": 10}
```
Result:
```
{"type":"feathers","data":[{"block_number":29582350,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art"},{"block_number":29554879,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554812,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554807,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554787,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::image::Wormhole Diagram::QmX9abc123"},{"block_number":29554703,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::image::Wormhole Diagram::QmX9abc123"}]}
```

Query with account_id:
```
{"type": "GetFeathers", "block_number": 0, "limit": 10, "account_id": "5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz"}
```
Result:
```
{"type":"feathers","data":[{"block_number":29582350,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art"},{"block_number":29554879,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554812,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554807,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554787,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::image::Wormhole Diagram::QmX9abc123"},{"block_number":29554703,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::image::Wormhole Diagram::QmX9abc123"}]}
```

Query with genre:
```
{"type": "GetFeathers", "block_number": 0, "limit": 10, "genre": "theory"}
```
Result:
```
{"type":"feathers","data":[{"block_number":29582350,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art"},{"block_number":29554879,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554812,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."},{"block_number":29554807,"index":2,"account_id":"5HQU5hQsdrmxV4sSqrShKnxxV7C8qLMKwzk6LRJas5Bpmxaz","remark":"FEATHER::theory::Onchain Telepathy::All governance is performance art..."}]}
```
