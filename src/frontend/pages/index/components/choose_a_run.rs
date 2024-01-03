use std::collections::BTreeMap;

use anyhow::Result;
use askama::Template;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_runs;
use crate::models::run::Run;
use crate::models::side::Side;

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
    ) -> Result<TemplateInstance> {
        let runs = get_runs(db).await?;

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

        Ok(TemplateInstance { runs })
    }
}
