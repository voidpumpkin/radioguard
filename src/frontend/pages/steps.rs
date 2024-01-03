use askama::Template;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderMap;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_step_data_uri_and_test_case_id;
use crate::db::get_test_case;
use crate::error::HttpResult;
use crate::models::side::Side;
use crate::services::compare_steps;

#[derive(Template)]
#[template(path = "frontend/pages/steps.jinja", escape = "none")]
struct TemplateInstance {
    list: Vec<ListItem>,
}

struct ListItem {
    unique_id: String,
    data_uri: String,
    cta: String,
    img_css: String,
}

async fn html_single(
    State(db): State<Pool<Sqlite>>,
    Path(step_id): Path<i64>,
) -> HttpResult<Html<String>> {
    let (data_uri, _) = get_step_data_uri_and_test_case_id(step_id, &db).await?;

    Ok(Html(
        TemplateInstance {
            list: vec![ListItem {
                unique_id: "single".to_string(),
                data_uri,
                cta: "🖼️".to_string(),
                img_css: "".to_string(),
            }],
        }
        .render()?,
    ))
}

async fn html_diff(
    State(db): State<Pool<Sqlite>>,
    Path((left_step_id, right_step_id)): Path<(i64, i64)>,
) -> HttpResult<(HeaderMap, impl IntoResponse)> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "public, max-age=31557600".parse()?);

    let (left_data_uri, left_test_case_id) =
        get_step_data_uri_and_test_case_id(left_step_id, &db).await?;
    let left_test_case = get_test_case(&db, left_test_case_id).await?;
    let (right_data_uri, right_test_case_id) =
        get_step_data_uri_and_test_case_id(right_step_id, &db).await?;
    let right_test_case = get_test_case(&db, right_test_case_id).await?;
    let ignore_ranges = [left_test_case.ignore_ranges, right_test_case.ignore_ranges].concat();

    let (contains_changes, diff_data_uri) = compare_steps(
        left_data_uri.as_str(),
        right_data_uri.as_str(),
        &ignore_ranges,
    )
    .await?;

    let mut list = vec![];
    list.push(ListItem {
        unique_id: Side::Left.to_string(),
        data_uri: left_data_uri,
        cta: "👈".to_string(),
        img_css: "".to_string(),
    });
    if contains_changes {
        list.push(ListItem {
            unique_id: "diff".to_string(),
            data_uri: diff_data_uri,
            cta: "🤝".to_string(),
            img_css: "invert".to_string(),
        });
    }
    list.push(ListItem {
        unique_id: Side::Right.to_string(),
        data_uri: right_data_uri,
        cta: "👉".to_string(),
        img_css: "".to_string(),
    });
    Ok((headers, Html(TemplateInstance { list }.render()?)))
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/:step_id", get(html_single))
        .route("/:left_step_id/:right_step_id", get(html_diff))
        .with_state(db)
}
