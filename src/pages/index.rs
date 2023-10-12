use std::collections::BTreeMap;

use askama::Template;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use serde::Deserialize;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::components::choose_a_run;
use crate::models::side::Side;

#[derive(Template)]
#[template(path = "pages/index.jinja", escape = "none")]
struct TemplateInstance {
    left: String,
    right: String,
}

#[derive(Deserialize)]
pub struct QueryParams {
    left_run: Option<i64>,
    right_run: Option<i64>,
}

pub async fn html(
    State(db): State<Pool<Sqlite>>,
    Query(query_params): Query<BTreeMap<String, String>>,
    Query(QueryParams {
        left_run,
        right_run,
    }): Query<QueryParams>,
) -> impl IntoResponse {
    let left = if let Some(run_id) = left_run {
        format!("{run_id}")
    } else {
        choose_a_run::TemplateInstance::new(db.clone(), Side::Left, query_params.clone())
            .await
            .render()
            .unwrap()
    };

    let right = if let Some(run_id) = right_run {
        format!("{run_id}")
    } else {
        choose_a_run::TemplateInstance::new(db.clone(), Side::Right, query_params.clone())
            .await
            .render()
            .unwrap()
    };

    Html(TemplateInstance { left, right }.render().unwrap())
}
