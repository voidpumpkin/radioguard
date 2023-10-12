pub mod components;
pub mod models;
pub mod pages;

use axum::{routing::get, Router};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = SqliteConnectOptions::new()
        // .filename(":memory:")
        .filename("data.db")
        .create_if_missing(true);
    let db = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .route("/", get(pages::index::html))
        .with_state(db)
        .nest("/dist", axum_static::static_router("dist"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
