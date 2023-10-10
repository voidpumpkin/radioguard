pub mod models;
pub mod templates;

use axum::{routing::get, Router};
use std::net::SocketAddr;

use crate::templates::pages::index::handle_page_index;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handle_page_index));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
