use askama::Template;
use axum::extract::State;
use axum::response::Html;
use sqlx::Pool;
use sqlx::Sqlite;

#[derive(Template)]
#[template(path = "templates/pages/index.jinja")]
struct TemplateInstance<'a> {
    name: &'a str,
}

pub async fn handle_page_index(State(db): State<Pool<Sqlite>>) -> Html<String> {
    Html(TemplateInstance { name: "wor1d" }.render().unwrap())
}
