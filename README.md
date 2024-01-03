# Radioguard

Visual Testing

## Prep

```
npm i
cargo install cargo-watch

cargo bin sqlx database create
cargo bin sqlx migrate run
```

## Run

```
cargo bin cargo-watch -s "npm run build && cargo run"
```
