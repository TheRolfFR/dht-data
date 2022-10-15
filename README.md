# DHT-DATA

DHT sensor data logger written with rust

## Tech
- Rust
- Reqwest +[blocking, json]: decoding and sync behaviour
- serde +[derive]: serialize and deserialize data
- serde_json: bruh
- rouille: extremely fast web micro-framework server
- chrono +[serde]: dates manipulation
- chrono-tz: Europe/Paris timezone
- dirs: Get the fucking home directory

## Install
Please read instruction provided by this [link](https://letmegooglethat.com/?q=how+to+clone+github+repository).

## Compile
```sh
cargo build
```

## Run
```sh
cargo run -- <URL_OF_SENSOR> [<DEFAULT_PATH> <DEFAULT_PORT>]
```

## Contribute
```rust
const contribute: bool = false;
```

## Releases
```rust
const releases: Option<Vec<Link>> = None
```