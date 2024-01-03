use async_recursion::async_recursion;
use chrono::NaiveDateTime;
use chrono::Utc;
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

    for run in runs_untagged.into_iter() {
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
            name: run.name,
            created_at: NaiveDateTime::parse_from_str(&run.created_at, "%F %T").unwrap(),
            tags,
        })
    }
    runs
}

pub async fn insert_and_get_run(db: &Pool<Sqlite>, name: &str, tag_values: &[String]) -> Run {
    let now = Utc::now().naive_utc().to_string();

    sqlx::query!(
        "
    INSERT INTO run(name,created_at)
    VALUES ($1, $2);
                ",
        name,
        now
    )
    .execute(db)
    .await
    .ok();

    let run = sqlx::query!(
        "
    SELECT *
    FROM run
    WHERE run.name = ?
            ",
        name,
    )
    .fetch_one(db)
    .await
    .unwrap();

    let mut tags = vec![];
    for tag in tag_values {
        let tag = insert_and_get_tag(db, tag).await;

        sqlx::query!(
            "
        INSERT INTO run_tag(run_id,tag_id)
        VALUES ($1, $2);
                    ",
            run.id,
            tag.id
        )
        .execute(db)
        .await
        .ok();

        tags.push(tag);
    }

    Run {
        id: run.id,
        name: run.name,
        created_at: NaiveDateTime::parse_from_str(&run.created_at, "%F %T").unwrap(),
        tags,
    }
}

pub async fn insert_and_get_tag(db: &Pool<Sqlite>, tag: &str) -> Tag {
    sqlx::query!(
        "
    INSERT INTO tag(value)
    VALUES ($1);
                ",
        tag,
    )
    .execute(db)
    .await
    .ok();

    sqlx::query!(
        "
    SELECT *
    FROM tag
    WHERE tag.value = ?;
                ",
        tag,
    )
    .map(|row| Tag {
        id: row.id,
        value: row.value,
    })
    .fetch_one(db)
    .await
    .unwrap()
}

pub async fn insert_and_get_test_case(db: &Pool<Sqlite>, run_id: i64, name: &str) -> TestCase {
    let now = Utc::now().naive_utc().to_string();

    sqlx::query!(
        "
    INSERT INTO test_case(run_id,name,created_at)
    VALUES (?, ?, ?);
                ",
        run_id,
        name,
        now
    )
    .execute(db)
    .await
    .ok();

    let test_case = sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE test_case.name = ? and run_id = ?
            ",
        name,
        run_id
    )
    .fetch_one(db)
    .await
    .unwrap();

    TestCase {
        id: test_case.id,
        run_id,
        name: test_case.name,
        created_at: NaiveDateTime::parse_from_str(&test_case.created_at, "%F %T").unwrap(),
    }
}

pub async fn insert_and_get_step(
    db: &Pool<Sqlite>,
    test_case_id: i64,
    name: &str,
    img_base64_url: &str,
    parent_step_id: Option<i64>,
) -> Step {
    let now = Utc::now().naive_utc().to_string();

    sqlx::query!(
        "
    INSERT INTO step(test_case_id,parent_step_id,name,created_at,data_uri)
    VALUES (?, ?, ?, ?, ?);
                ",
        test_case_id,
        parent_step_id,
        name,
        now,
        img_base64_url
    )
    .execute(db)
    .await
    .ok();

    let step = sqlx::query!(
        "
    SELECT *
    FROM step
    WHERE step.name = ? and test_case_id = ?
            ",
        name,
        test_case_id
    )
    .fetch_one(db)
    .await
    .unwrap();

    let children_steps = get_steps(db, test_case_id, step.id.into()).await;

    Step {
        id: step.id,
        name: step.name,
        test_case_id,
        data_uri: step.data_uri,
        created_at: NaiveDateTime::parse_from_str(&step.created_at, "%F %T").unwrap(),
        children_steps,
    }
}
