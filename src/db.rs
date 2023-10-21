use async_recursion::async_recursion;
use chrono::NaiveDateTime;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::run::Run;
use crate::models::step::Step;
use crate::models::tag::Tag;
use crate::models::test_case::TestCase;
use crate::models::test_case::TestCaseWithSteps;

pub async fn get_step_data_uri(id: i64, db: &Pool<Sqlite>) -> String {
    sqlx::query!(
        "
    SELECT data_uri
    FROM step
    WHERE id is $1
            ",
        id
    )
    .map(|row| row.data_uri)
    .fetch_one(db)
    .await
    .unwrap()
}

#[async_recursion]
pub async fn get_steps(
    db: &Pool<Sqlite>,
    left_test_case: i64,
    parent_step_id: Option<i64>,
) -> Vec<Step> {
    let mut steps = sqlx::query!(
        "
    SELECT *
    FROM step
    WHERE step.test_case_id is $1 and step.parent_step_id is $2
            ",
        left_test_case,
        parent_step_id
    )
    .map(|row| Step {
        id: row.id,
        name: row.name,
        data_uri: row.data_uri,
        created_at: NaiveDateTime::parse_from_str(&row.created_at, "%F %T").unwrap(),
        test_case_id: row.test_case_id,
        children_steps: vec![],
    })
    .fetch_all(db)
    .await
    .unwrap();

    for step in steps.iter_mut() {
        step.children_steps = get_steps(db, left_test_case, step.id.into()).await;
    }

    steps
}

pub async fn get_run_test_cases(db: &Pool<Sqlite>, run_id: i64) -> Vec<TestCase> {
    sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE run_id = $1
        ",
        run_id
    )
    .map(|row| TestCase {
        id: row.id,
        run_id: row.run_id,
        name: row.name,
        created_at: NaiveDateTime::parse_from_str(&row.created_at, "%F %T").unwrap(),
    })
    .fetch_all(db)
    .await
    .unwrap()
}

pub async fn get_case_with_steps(db: &Pool<Sqlite>, test_case_id: i64) -> TestCaseWithSteps {
    let steps = get_steps(db, test_case_id, None).await;

    let row = sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE test_case.id = $1
        ",
        test_case_id
    )
    .fetch_one(db)
    .await
    .unwrap();

    TestCaseWithSteps {
        id: row.id,
        run_id: row.run_id,
        name: row.name,
        created_at: NaiveDateTime::parse_from_str(&row.created_at, "%F %T").unwrap(),
        steps,
    }
}

pub async fn get_runs(db: Pool<Sqlite>) -> Vec<Run> {
    let mut runs = vec![];

    let runs_untagged = sqlx::query!(
        "
    SELECT *
    FROM run
            ",
    )
    .fetch_all(&db)
    .await
    .unwrap();

    for run in runs_untagged.iter() {
        let tags = sqlx::query!(
            "
    SELECT tag.*
    FROM tag
    JOIN run_tag ON run_tag.tag_id = tag.id
    WHERE run_id = ?;
                ",
            run.id,
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
    runs
}
