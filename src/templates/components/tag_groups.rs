use askama::Template;
use axum::extract::State;
use axum::response::Html;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::TagGroup;

#[derive(Template)]
#[template(path = "templates/components/tag_groups.jinja")]
struct TemplateInstance {
    tag_groups: Vec<TagGroup>,
}

pub async fn handle_component_tag_groups(State(db): State<Pool<Sqlite>>) -> Html<String> {
    let tag_groups = sqlx::query!(
        "
SELECT *
FROM tag_group
        "
    )
    .map(|e| TagGroup {
        id: e.id,
        name: e.name,
    })
    .fetch_all(&db)
    .await
    .unwrap();

    Html(TemplateInstance { tag_groups }.render().unwrap())
}
