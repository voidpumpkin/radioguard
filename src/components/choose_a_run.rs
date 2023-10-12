use std::collections::BTreeMap;

use askama::Template;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::db::run::Run;
use crate::models::db::tag::Tag;
use crate::models::side::Side;

#[derive(Template)]
#[template(path = "components/choose_a_run.jinja")]
pub struct TemplateInstance {
    runs: Vec<(Run, String)>,
}

impl TemplateInstance {
    pub async fn new(
        db: Pool<Sqlite>,
        side: Side,
        query_params: BTreeMap<String, String>,
    ) -> TemplateInstance {
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

        let runs = runs
            .into_iter()
            .map(|run| {
                let mut query_params = query_params.clone();
                query_params.insert(format!("{side}_run"), run.id.to_string());
                let link = query_params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join("&");

                (run, link)
            })
            .collect();

        TemplateInstance { runs }
    }
}
