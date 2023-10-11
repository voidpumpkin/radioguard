use askama::Template;
use axum::extract::State;
use axum::response::Html;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::run::Run;
use crate::models::tag::Tag;

#[derive(Template)]
#[template(path = "pages/index.jinja")]
struct TemplateInstance {
    runs: Vec<Run>,
}

pub async fn handle_page_index(State(db): State<Pool<Sqlite>>) -> Html<String> {
    let runs_untagged = sqlx::query!(
        "
SELECT *
FROM run
        "
    )
    .map(|row| (row.id, row.created_at))
    .fetch_all(&db)
    .await
    .unwrap();

    let mut runs = vec![];

    for run in runs_untagged.iter() {
        let tags = sqlx::query!(
            "
SELECT tag.*
FROM tag
JOIN run_tag ON run_tag.tag_id = tag.id
WHERE run_id = ?;
            ",
            run.0
        )
        .map(|row| Tag {
            id: row.id,
            value: row.value,
        })
        .fetch_all(&db)
        .await
        .unwrap();

        runs.push(Run {
            id: run.0,
            created_at: NaiveDateTime::parse_from_str(&run.1, "%F %T").unwrap(),
            tags,
        })
    }

    Html(TemplateInstance { runs }.render().unwrap())
}
