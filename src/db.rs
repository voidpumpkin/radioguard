use async_recursion::async_recursion;
use chrono::Utc;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::run::Run;
use crate::models::step::Step;
use crate::models::tag::Tag;
use crate::models::test_case::TestCase;
use crate::models::test_case::TestCaseWithSteps;
use anyhow::Result;

pub async fn get_step_data_uri_and_test_case_id(
    id: i64,
    db: &Pool<Sqlite>,
) -> Result<(String, i64)> {
    Ok(sqlx::query!(
        "
    SELECT data_uri, test_case_id
    FROM step
    WHERE id is $1
            ",
        id
    )
    .map(|row| (row.data_uri, row.test_case_id))
    .fetch_one(db)
    .await?)
}

#[async_recursion]
pub async fn get_steps(
    db: &Pool<Sqlite>,
    left_test_case: i64,
    parent_step_id: Option<i64>,
) -> Result<Vec<Step>> {
    let mut steps = sqlx::query!(
        "
    SELECT *
    FROM step
    WHERE step.test_case_id is $1 and step.parent_step_id is $2
            ",
        left_test_case,
        parent_step_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Step {
            id: row.id,
            name: row.name,
            data_uri: row.data_uri,
            created_at: row.created_at.parse()?,
            test_case_id: row.test_case_id,
            children_steps: vec![],
        })
    })
    .collect::<Result<Vec<_>>>()?;

    for step in steps.iter_mut() {
        step.children_steps = get_steps(db, left_test_case, step.id.into()).await?;
    }

    Ok(steps)
}

pub async fn get_run_test_cases(db: &Pool<Sqlite>, run_id: i64) -> Result<Vec<TestCase>> {
    let vec = sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE run_id = $1
        ",
        run_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| {
        Ok(TestCase {
            id: row.id,
            run_id: row.run_id,
            name: row.name,
            ignore_areas: serde_json::from_str(row.ignore_areas.as_str())?,
            created_at: row.created_at.parse()?,
        })
    })
    .collect::<Result<Vec<_>>>()?;
    Ok(vec)
}

pub async fn get_test_case(db: &Pool<Sqlite>, test_case_id: i64) -> Result<TestCase> {
    let row = sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE id = $1
        ",
        test_case_id
    )
    .fetch_one(db)
    .await?;
    Ok(TestCase {
        id: row.id,
        run_id: row.run_id,
        name: row.name,
        ignore_areas: serde_json::from_str(row.ignore_areas.as_str())?,
        created_at: row.created_at.parse()?,
    })
}

pub async fn get_case_with_steps(
    db: &Pool<Sqlite>,
    test_case_id: i64,
) -> Result<TestCaseWithSteps> {
    let steps = get_steps(db, test_case_id, None).await?;

    let row = sqlx::query!(
        "
    SELECT *
    FROM test_case
    WHERE test_case.id = $1
        ",
        test_case_id
    )
    .fetch_one(db)
    .await?;

    Ok(TestCaseWithSteps {
        id: row.id,
        run_id: row.run_id,
        name: row.name,
        created_at: row.created_at.parse()?,
        steps,
    })
}

pub async fn get_runs(db: Pool<Sqlite>) -> Result<Vec<Run>> {
    let mut runs = vec![];

    let runs_untagged = sqlx::query!(
        "
    SELECT *
    FROM run
            ",
    )
    .fetch_all(&db)
    .await?;

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
        .await?;

        runs.push(Run {
            id: run.id,
            name: run.name,
            created_at: run.created_at.parse()?,
            tags,
        })
    }
    Ok(runs)
}

pub async fn insert_and_get_run(
    db: &Pool<Sqlite>,
    name: &str,
    tag_values: &[String],
) -> Result<Run> {
    let now = Utc::now().to_string();

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
    .await?;

    let mut tags = vec![];
    for tag in tag_values {
        let tag = insert_and_get_tag(db, tag).await?;

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

    Ok(Run {
        id: run.id,
        name: run.name,
        created_at: run.created_at.parse()?,
        tags,
    })
}

pub async fn insert_and_get_tag(db: &Pool<Sqlite>, tag: &str) -> Result<Tag> {
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

    Ok(sqlx::query!(
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
    .await?)
}

pub async fn insert_and_get_test_case(
    db: &Pool<Sqlite>,
    run_id: i64,
    name: &str,
    ignore_areas: Vec<((u32, u32), (u32, u32))>,
) -> Result<TestCase> {
    let now = Utc::now().to_string();
    let ignore_areas = serde_json::to_string(&ignore_areas)?;

    sqlx::query!(
        "
    INSERT INTO test_case(run_id,name,created_at,ignore_areas)
    VALUES (?, ?, ?, ?);
                ",
        run_id,
        name,
        now,
        ignore_areas
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
    .await?;

    Ok(TestCase {
        id: test_case.id,
        run_id,
        name: test_case.name,
        ignore_areas: serde_json::from_str(test_case.ignore_areas.as_str())?,
        created_at: test_case.created_at.parse()?,
    })
}

pub async fn insert_and_get_step(
    db: &Pool<Sqlite>,
    test_case_id: i64,
    name: &str,
    img_base64_url: &str,
    parent_step_id: Option<i64>,
) -> Result<Step> {
    let now = Utc::now().to_string();

    sqlx::query!(
        "
    INSERT INTO step(test_case_id,parent_step_id,name,created_at,data_uri)
    VALUES (?, ?, ?, ?, ?);
                ",
        test_case_id,
        parent_step_id,
        name,
        now,
        img_base64_url,
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
    .await?;

    let children_steps = get_steps(db, test_case_id, step.id.into()).await?;

    Ok(Step {
        id: step.id,
        name: step.name,
        test_case_id,
        data_uri: step.data_uri,
        created_at: step.created_at.parse()?,
        children_steps,
    })
}
