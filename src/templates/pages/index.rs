use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "templates/pages/index.jinja")]
pub struct HelloTemplate<'a> {
    pub name: &'a str,
}

pub async fn handle_page_index() -> Html<String> {
    Html(HelloTemplate { name: "world" }.render().unwrap())
}
