use std::collections::BTreeMap;

use askama::Template;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::side::Side;
use crate::models::tag::Tag;
use crate::models::test_case::TestCase;

#[derive(Template)]
#[template(path = "frontend/pages/runs/components/test_cases.jinja")]
pub struct TemplateInstance {
    test_cases: Vec<(TestCase, String)>,
}

impl TemplateInstance {
    pub async fn new(
        db: Pool<Sqlite>,
        side: Side,
        query_params: BTreeMap<String, String>,
        run_id: i64,
    ) -> TemplateInstance {
        let test_cases_untagged = sqlx::query!(
            "
    SELECT *
    FROM test_case
    WHERE run_id = ?
            ",
            run_id
        )
        .fetch_all(&db)
        .await
        .unwrap();

        let mut test_cases = vec![];

        for test_case in test_cases_untagged.into_iter() {
            let tags = sqlx::query!(
                "
    SELECT tag.*
    FROM tag
    JOIN test_case_tag ON test_case_tag.tag_id = tag.id
    WHERE test_case_id = ?;
                ",
                test_case.id
            )
            .map(|row| Tag {
                id: row.id,
                value: row.value,
            })
            .fetch_all(&db)
            .await
            .unwrap();

            test_cases.push(TestCase {
                id: test_case.id,
                created_at: NaiveDateTime::parse_from_str(&test_case.created_at, "%F %T").unwrap(),
                tags,
                run_id,
                name: test_case.name,
            })
        }

        let test_cases = test_cases
            .into_iter()
            .map(|test_case| {
                let mut query_params = query_params.clone();
                query_params.insert(format!("{side}_test_case"), test_case.id.to_string());
                query_params.remove(&format!("{side}_run"));
                let link = query_params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join("&");

                (test_case, link)
            })
            .collect();

        TemplateInstance { test_cases }
    }
}
