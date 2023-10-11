pub mod models;
pub mod templates;

use axum::{routing::get, Router};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::net::SocketAddr;

use crate::templates::pages::index::handle_page_index;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = SqliteConnectOptions::new()
        // .filename(":memory:")
        .filename("data.db")
        .create_if_missing(true);
    let db = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .route("/", get(handle_page_index))
        // .route("/tag_groups", get(handle_component_tag_groups))
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
