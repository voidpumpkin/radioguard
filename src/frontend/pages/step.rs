use askama::Template;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_step_data_uri;

#[derive(Template)]
#[template(path = "frontend/pages/step.jinja", escape = "none")]
struct TemplateInstance {
    data_uri: String,
}

async fn html(State(db): State<Pool<Sqlite>>, Path(id): Path<i64>) -> Html<String> {
    let data_uri = get_step_data_uri(id, &db).await;
    Html(TemplateInstance { data_uri }.render().unwrap())
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new().route("/:id", get(html)).with_state(db)
}
