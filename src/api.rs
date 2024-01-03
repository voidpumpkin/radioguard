use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_step_data_uri;
use crate::db::insert_and_get_run;
use crate::db::insert_and_get_step;
use crate::db::insert_and_get_test_case;
use crate::error::HttpResult;
use crate::services::compare_steps;

#[derive(Debug, Serialize)]
struct Comparison {
    contains_changes: bool,
}

async fn diff_steps_by_image(
    State(db): State<Pool<Sqlite>>,
    Path(path): Path<(Option<i64>, Option<i64>)>,
) -> HttpResult<(HeaderMap, impl IntoResponse)> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "public, max-age=31557600".parse()?);

    let (Some(left_step_id), Some(right_step_id)) = path else {
        return Ok((
            headers,
            Json(Comparison {
                contains_changes: false,
            }),
        ));
    };

    let left_data_uri = get_step_data_uri(left_step_id, &db).await?;
    let right_data_uri = get_step_data_uri(right_step_id, &db).await?;

    let (contains_changes, _diff_img_data_uri) =
        compare_steps(left_data_uri.as_str(), right_data_uri.as_str()).await?;

    Ok((headers, Json(Comparison { contains_changes })))
}

async fn post_run() {
    todo!()
}

async fn post_test_case() {
    todo!()
}

#[derive(Debug, Deserialize)]
struct PostStepReqBody {
    run_id: String,
    run_tags: Vec<String>,
    test_case_name: String,
    step_name: String,
    img_base64_url: String,
    parent_step_id: Option<i64>,
}

#[derive(Serialize)]
pub struct PostStepResBody {
    // None on errors
    pub step_id: Option<i64>,
}

async fn post_step(
    State(db): State<Pool<Sqlite>>,
    Json(body): Json<PostStepReqBody>,
) -> impl IntoResponse {
    let PostStepReqBody {
        run_id,
        run_tags,
        test_case_name,
        step_name,
        img_base64_url,
        parent_step_id,
    } = body;

    let run = match insert_and_get_run(&db, &run_id, &run_tags).await {
        Ok(run) => run,
        Err(err) => {
            log::error!("{err}");
            return Json(PostStepResBody { step_id: None });
        }
    };
    let test_case = match insert_and_get_test_case(&db, run.id, &test_case_name).await {
        Ok(test_case) => test_case,
        Err(err) => {
            log::error!("{err}");
            return Json(PostStepResBody { step_id: None });
        }
    };
    let step = match insert_and_get_step(
        &db,
        test_case.id,
        &step_name,
        &img_base64_url,
        parent_step_id,
    )
    .await
    {
        Ok(step) => step,
        Err(err) => {
            log::error!("{err}");
            return Json(PostStepResBody { step_id: None });
        }
    };

    Json(PostStepResBody {
        step_id: Some(step.id),
    })
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route(
            "/steps/:left_step_id/:right_step_id",
            get(diff_steps_by_image),
        )
        .route("/runs", post(post_run))
        .route("/test_cases", post(post_test_case))
        .route("/steps", post(post_step))
        .with_state(db)
}
