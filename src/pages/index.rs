use askama::Template;
use axum::extract::State;
use axum::response::Html;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

#[derive(Template)]
#[template(path = "pages/index.jinja")]
struct TemplateInstance {
    run_date_times: Vec<NaiveDateTime>,
}

pub async fn handle_page_index(State(db): State<Pool<Sqlite>>) -> Html<String> {
    let run_date_times = sqlx::query!(
        "
SELECT created_at
FROM run
        "
    )
    .map(|record| NaiveDateTime::parse_from_str(&record.created_at, "%F %T").unwrap())
    .fetch_all(&db)
    .await
    .unwrap();

    Html(TemplateInstance { run_date_times }.render().unwrap())
}
