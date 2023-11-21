# Radioguard

Visual Testing

## Prep

```
npm i
cargo install cargo-watch

sqlx database create
sqlx migrate run
```

## Run

```
cargo watch -s "npm run build && cargo run"
```
