#![deny(clippy::unwrap_used)]

pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod models;
pub mod services;

use axum::Router;
use frontend::pages;
use std::net::SocketAddr;

use dotenvy_macro::dotenv;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    pretty_env_logger::init();

    let options = SqliteConnectOptions::from_str(dotenv!("DATABASE_URL"))?.create_if_missing(true);
    let db = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .nest("/", pages::index::router(db.clone()))
        .nest("/runs", pages::runs::router(db.clone()))
        .nest("/steps", pages::steps::router(db.clone()))
        .nest("/api", api::router(db.clone()))
        .nest("/dist", axum_static::static_router("dist"));

    let addr = SocketAddr::from_str(dotenv!("ADDRESS"))?;
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
