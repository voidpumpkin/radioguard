use std::collections::BTreeMap;

use askama::Template;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::run::Run;
use crate::models::side::Side;
use crate::models::tag::Tag;

#[derive(Template)]
#[template(path = "frontend/pages/index/components/choose_a_run.jinja")]
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
                run.id
            )
            .map(|row| Tag {
                id: row.id,
                value: row.value,
            })
            .fetch_all(&db)
            .await
            .unwrap();

            runs.push(Run {
                id: run.id,
                created_at: NaiveDateTime::parse_from_str(&run.created_at, "%F %T").unwrap(),
                tags,
            })
        }

        let runs = runs
            .into_iter()
            .filter_map(|run| {
                let run_id = run.id.to_string();
                let mut query_params = query_params.clone();

                let link = match query_params
                    .get(&format!("{}_run", side.opposite()))
                    .cloned()
                {
                    Some(other_run_id) if other_run_id == run_id => {
                        return None;
                    }
                    Some(other_run_id) => {
                        let (left_run_id, right_run_id) = if side == Side::Left {
                            (run_id.to_string(), other_run_id)
                        } else {
                            (other_run_id, run_id.to_string())
                        };

                        format!("/runs/{left_run_id}/{right_run_id}")
                    }
                    None => {
                        query_params.insert(format!("{side}_run"), run_id);
                        let query_params_joined = query_params
                            .iter()
                            .map(|(k, v)| format!("{k}={v}"))
                            .collect::<Vec<String>>()
                            .join("&");
                        format!("?{query_params_joined}")
                    }
                };

                Some((run, link))
            })
            .collect();

        TemplateInstance { runs }
    }
}
