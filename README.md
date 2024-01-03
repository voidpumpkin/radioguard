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
cargo run watch -s "npm run build && cargo run"
```
