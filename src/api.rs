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

use crate::db::get_step_data_uri_and_test_case_id;
use crate::db::get_test_case;
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

    let (left_data_uri, left_test_case_id) =
        get_step_data_uri_and_test_case_id(left_step_id, &db).await?;
    let left_test_case = get_test_case(&db, left_test_case_id).await?;
    let (right_data_uri, right_test_case_id) =
        get_step_data_uri_and_test_case_id(right_step_id, &db).await?;
    let right_test_case = get_test_case(&db, right_test_case_id).await?;
    let ignore_ranges = [left_test_case.ignore_areas, right_test_case.ignore_areas].concat();

    let (contains_changes, _diff_img_data_uri) = compare_steps(
        left_data_uri.as_str(),
        right_data_uri.as_str(),
        &ignore_ranges,
    )
    .await?;

    Ok((headers, Json(Comparison { contains_changes })))
}

#[derive(Debug, Deserialize)]
struct PostStepReqBody {
    run_id: String,
    run_tags: Vec<String>,
    test_case_name: String,
    step_name: String,
    img_base64_url: String,
    parent_step_id: Option<i64>,
    ignore_areas: Vec<((u32, u32), (u32, u32))>,
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
        ignore_areas,
    } = body;

    let run = match insert_and_get_run(&db, &run_id, &run_tags).await {
        Ok(run) => run,
        Err(err) => {
            log::error!("{err}");
            return Json(PostStepResBody { step_id: None });
        }
    };
    let test_case = match insert_and_get_test_case(&db, run.id, &test_case_name, ignore_areas).await
    {
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
        .route("/steps", post(post_step))
        .with_state(db)
}
