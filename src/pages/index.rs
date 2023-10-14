use std::collections::BTreeMap;

use askama::Template;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
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

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    left_run: Option<i64>,
    right_run: Option<i64>,
}

async fn side(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    side: Side,
    run_id: Option<i64>,
) -> String {
    if let Some(run_id) = run_id {
        format!("{run_id}")
    } else {
        choose_a_run::TemplateInstance::new(db, side, all_query_params)
            .await
            .render()
            .unwrap()
    }
}

pub async fn html(
    State(db): State<Pool<Sqlite>>,
    Query(all_qp): Query<BTreeMap<String, String>>,
    Query(QueryParams {
        left_run,
        right_run,
    }): Query<QueryParams>,
) -> Html<String> {
    let left = side(db.clone(), all_qp.clone(), Side::Left, left_run).await;
    let right = side(db.clone(), all_qp.clone(), Side::Right, right_run).await;
    Html(TemplateInstance { left, right }.render().unwrap())
}
