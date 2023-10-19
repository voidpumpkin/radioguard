use std::fmt::Write;

use askama::Template;
use async_recursion::async_recursion;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use chrono::NaiveDateTime;
use similar::TextDiff;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::models::step::Step;
use crate::models::test_case::TestCase;

#[derive(Template)]
#[template(path = "frontend/pages/runs.jinja", escape = "none")]
struct TemplateInstance {
    diff: String,
}

fn write_in_steps(w: &mut String, steps: &[Step], ident: usize) {
    for step in steps {
        for _ in 0..ident {
            write!(w, "    ").unwrap();
        }
        writeln!(w, "{}", step.name).unwrap();
        write_in_steps(w, &step.children_steps, ident + 1);
    }
}

#[async_recursion]
async fn get_steps(
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

async fn get_run_test_cases(db: &Pool<Sqlite>, run_id: i64) -> Vec<TestCase> {
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

async fn get_case_string(db: &Pool<Sqlite>, test_case_id: i64) -> String {
    let test_case = sqlx::query!(
        "
SELECT *
FROM test_case
WHERE test_case.id = $1
        ",
        test_case_id
    )
    .map(|row| TestCase {
        id: row.id,
        run_id: row.run_id,
        name: row.name,
        created_at: NaiveDateTime::parse_from_str(&row.created_at, "%F %T").unwrap(),
    })
    .fetch_one(db)
    .await
    .unwrap();

    let steps = get_steps(db, test_case.id, None).await;
    let mut result = "".to_string();
    write_in_steps(&mut result, &steps, 0);
    result
}

pub async fn html(
    State(db): State<Pool<Sqlite>>,
    Path((left_run, right_run)): Path<(i64, i64)>,
) -> Html<String> {
    let left_cases = get_run_test_cases(&db, left_run).await;
    let mut right_cases = get_run_test_cases(&db, right_run).await;

    let mut matches: Vec<(TestCase, TestCase)> = vec![];
    let mut left_loners: Vec<TestCase> = vec![];

    for l in left_cases.iter() {
        if let Some(pos) = right_cases.iter_mut().position(|r| l.name == r.name) {
            let r = right_cases.remove(pos);
            matches.push((l.clone(), r));
        } else {
            left_loners.push(l.clone());
        }
    }

    let right_loners: Vec<TestCase> = right_cases;

    let mut diffs: String = String::default();

    for test_case in left_loners.into_iter() {
        let content = get_case_string(&db, test_case.id).await;

        let mut hunk = String::default();
        writeln!(&mut hunk, "--- {}", test_case.name).unwrap();
        writeln!(&mut hunk, "+++ {}", test_case.name).unwrap();
        writeln!(&mut hunk, "@@ -1,{} @@", content.len()).unwrap();
        for line in content.lines() {
            writeln!(&mut hunk, "- {line}").unwrap();
        }

        diffs += hunk.to_string().as_str();
    }

    for test_case in right_loners.into_iter() {
        let content = get_case_string(&db, test_case.id).await;

        let mut hunk = String::default();
        writeln!(&mut hunk, "--- {}", test_case.name).unwrap();
        writeln!(&mut hunk, "+++ {}", test_case.name).unwrap();
        writeln!(&mut hunk, "@@ +1,{} @@", content.len()).unwrap();
        for line in content.lines() {
            writeln!(&mut hunk, "+ {line}").unwrap();
        }

        diffs += hunk.to_string().as_str();
    }

    for (left_test_case, right_test_case) in matches.into_iter() {
        let mut hunk = String::default();

        let l = get_case_string(&db, left_test_case.id).await;
        let r = get_case_string(&db, right_test_case.id).await;

        if l == r {
            writeln!(&mut hunk, "--- {}", left_test_case.name).unwrap();
            writeln!(&mut hunk, "+++ {}", right_test_case.name).unwrap();
            writeln!(&mut hunk, "@@ @@").unwrap();
            for line in l.lines() {
                writeln!(&mut hunk, " {line}").unwrap();
            }
            diffs += hunk.to_string().as_str();
            continue;
        }

        let text_diff = TextDiff::from_lines(&l, &r);

        let text_diff_ref = &text_diff;

        for text_hunk in text_diff
            .grouped_ops(usize::MAX / 2)
            .into_iter()
            .map(move |ops| similar::udiff::UnifiedDiffHunk::new(ops, text_diff_ref, false))
        {
            write!(&mut hunk, "{text_hunk}").unwrap();
        }

        diffs += hunk.to_string().as_str();
    }

    Html(TemplateInstance { diff: diffs }.render().unwrap())
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/:left_run_id/:right_run_id", get(html))
        .with_state(db)
}
