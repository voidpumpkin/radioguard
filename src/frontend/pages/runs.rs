pub mod components;

use std::collections::BTreeMap;

use askama::Template;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::side::Side;

use self::components::test_cases;

#[derive(Template)]
#[template(path = "frontend/pages/runs.jinja", escape = "none")]
struct TemplateInstance {
    left: String,
    right: String,
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    left_test_case: Option<i64>,
    right_test_case: Option<i64>,
}

async fn side(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    side: Side,
    test_case_id: Option<i64>,
    run_id: i64,
) -> String {
    if let Some(test_case_id) = test_case_id {
        format!("{test_case_id}")
    } else {
        test_cases::TemplateInstance::new(db, side, all_query_params, run_id)
            .await
            .render()
            .unwrap()
    }
}

pub async fn html(
    State(db): State<Pool<Sqlite>>,
    Query(all_qp): Query<BTreeMap<String, String>>,
    Query(QueryParams {
        left_test_case,
        right_test_case,
    }): Query<QueryParams>,
    Path((left_run, right_run)): Path<(i64, i64)>,
) -> Html<String> {
    let left = side(
        db.clone(),
        all_qp.clone(),
        Side::Left,
        left_test_case,
        left_run,
    )
    .await;

    let right = side(
        db.clone(),
        all_qp.clone(),
        Side::Right,
        right_test_case,
        right_run,
    )
    .await;

    Html(TemplateInstance { left, right }.render().unwrap())
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new().route("/", get(html)).with_state(db)
}
