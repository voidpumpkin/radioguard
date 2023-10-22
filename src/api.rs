use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use sqlx::Pool;
use sqlx::Sqlite;

async fn diff_steps_by_image(
    State(db): State<Pool<Sqlite>>,
    Path((left_step_id, right_step_id)): Path<(i64, i64)>,
) -> Html<String> {
    Html(format!(""))
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route(
            "api/steps/:left_step_id/:right_step_id",
            get(diff_steps_by_image),
        )
        .with_state(db)
}
