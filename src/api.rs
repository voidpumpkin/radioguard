use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_step_data_uri;
use crate::services::compare_steps;

#[derive(Debug, Serialize)]
struct Comparison {
    contains_changes: bool,
}

async fn diff_steps_by_image(
    State(db): State<Pool<Sqlite>>,
    Path(path): Path<(Option<i64>, Option<i64>)>,
) -> (HeaderMap, impl IntoResponse) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31557600".parse().unwrap(),
    );

    let (Some(left_step_id), Some(right_step_id)) = path else {
        return (
            headers,
            Json(Comparison {
                contains_changes: false,
            }),
        );
    };

    let left_data_uri = get_step_data_uri(left_step_id, &db).await;
    let right_data_uri = get_step_data_uri(right_step_id, &db).await;

    let (contains_changes, _diff_img_data_uri) =
        compare_steps(left_data_uri.as_str(), right_data_uri.as_str()).await;

    (headers, Json(Comparison { contains_changes }))
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route(
            "/steps/:left_step_id/:right_step_id",
            get(diff_steps_by_image),
        )
        .with_state(db)
}
