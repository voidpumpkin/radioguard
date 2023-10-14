use std::collections::BTreeMap;

use askama::Template;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use serde::Deserialize;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::components::choose_a_run;
use crate::components::test_cases;
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
    left_test_case: Option<i64>,
    right_run: Option<i64>,
    right_test_case: Option<i64>,
}

async fn diff_test_cases_html(left_test_case: i64, right_test_case: i64) -> Html<String> {
    let left = format!("{left_test_case}");
    let right = format!("{right_test_case}");
    Html(TemplateInstance { left, right }.render().unwrap())
}

async fn pick_right_test_case_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    left_test_case: i64,
    right_run: i64,
) -> Html<String> {
    let left = format!("{left_test_case}");
    let right = test_cases::TemplateInstance::new(
        db.clone(),
        Side::Right,
        all_query_params.clone(),
        right_run,
    )
    .await
    .render()
    .unwrap();
    Html(TemplateInstance { left, right }.render().unwrap())
}

async fn pick_left_test_case_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    right_test_case: i64,
    left_run: i64,
) -> Html<String> {
    let left = test_cases::TemplateInstance::new(
        db.clone(),
        Side::Left,
        all_query_params.clone(),
        left_run,
    )
    .await
    .render()
    .unwrap();
    let right = format!("{right_test_case}");
    Html(TemplateInstance { left, right }.render().unwrap())
}
async fn pick_test_cases_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    left_run: i64,
    right_run: i64,
) -> Html<String> {
    let left = test_cases::TemplateInstance::new(
        db.clone(),
        Side::Left,
        all_query_params.clone(),
        left_run,
    )
    .await
    .render()
    .unwrap();
    let right = test_cases::TemplateInstance::new(
        db.clone(),
        Side::Right,
        all_query_params.clone(),
        right_run,
    )
    .await
    .render()
    .unwrap();
    Html(TemplateInstance { left, right }.render().unwrap())
}
async fn pick_right_run_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    left_run: i64,
) -> Html<String> {
    let left = format!("{left_run}");
    let right =
        choose_a_run::TemplateInstance::new(db.clone(), Side::Right, all_query_params.clone())
            .await
            .render()
            .unwrap();
    Html(TemplateInstance { left, right }.render().unwrap())
}

async fn pick_left_run_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
    right_run: i64,
) -> Html<String> {
    let left =
        choose_a_run::TemplateInstance::new(db.clone(), Side::Left, all_query_params.clone())
            .await
            .render()
            .unwrap();
    let right = format!("{right_run}");
    Html(TemplateInstance { left, right }.render().unwrap())
}

async fn pick_runs_html(
    db: Pool<Sqlite>,
    all_query_params: BTreeMap<String, String>,
) -> Html<String> {
    let left =
        choose_a_run::TemplateInstance::new(db.clone(), Side::Left, all_query_params.clone())
            .await
            .render()
            .unwrap();
    let right =
        choose_a_run::TemplateInstance::new(db.clone(), Side::Right, all_query_params.clone())
            .await
            .render()
            .unwrap();
    Html(TemplateInstance { left, right }.render().unwrap())
}

pub async fn html(
    State(db): State<Pool<Sqlite>>,
    Query(all_qp): Query<BTreeMap<String, String>>,
    Query(QueryParams {
        left_test_case,
        right_test_case,
        left_run,
        right_run,
    }): Query<QueryParams>,
) -> Html<String> {
    match (left_test_case, right_test_case, left_run, right_run) {
        (Some(left_test_case), Some(right_test_case), None, None) => {
            diff_test_cases_html(left_test_case, right_test_case).await
        }
        (Some(left_test_case), None, None, Some(right_run)) => {
            pick_right_test_case_html(db, all_qp, left_test_case, right_run).await
        }
        (None, Some(right_test_case), Some(left_run), None) => {
            pick_left_test_case_html(db, all_qp, right_test_case, left_run).await
        }
        (None, None, Some(left_run), Some(right_run)) => {
            pick_test_cases_html(db, all_qp, left_run, right_run).await
        }
        (None, None, Some(left_run), None) => pick_right_run_html(db, all_qp, left_run).await,
        (None, None, None, Some(right_run)) => pick_left_run_html(db, all_qp, right_run).await,
        (None, None, None, None) => pick_runs_html(db, all_qp).await,
        // Baddies
        _ => Html(dbg!(
            "Correct your query params, something is incorrect".to_string()
        )),
    }
}
