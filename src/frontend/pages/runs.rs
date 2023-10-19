use std::collections::HashMap;
use std::fmt::Write;
use std::ops::Add;
use std::ops::AddAssign;

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
use velcro::hash_map;

use crate::models::side::Side;
use crate::models::step::Step;
use crate::models::test_case::TestCase;
use crate::models::test_case::TestCaseWithSteps;

#[derive(Template)]
#[template(path = "frontend/pages/runs.jinja", escape = "none")]
struct TemplateInstance {
    raw_templates: String,
    diff: String,
    map: String,
}

fn write_in_steps(
    w: &mut String,
    line_id_map: &mut HashMap<usize, i64>,
    line: &mut usize,
    steps: &[Step],
    ident: usize,
) {
    for step in steps {
        for _ in 0..ident {
            write!(w, "    ").unwrap();
        }
        writeln!(w, "{}", step.name).unwrap();
        line.add_assign(1);
        line_id_map.insert(*line, step.id);
        write_in_steps(w, line_id_map, line, &step.children_steps, ident + 1);
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

async fn get_case(db: &Pool<Sqlite>, test_case_id: i64) -> TestCaseWithSteps {
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

fn case_to_string(test_case: &TestCaseWithSteps) -> (String, HashMap<usize, i64>) {
    let mut result = "".to_string();
    let mut line_id_map = Default::default();
    let mut line = 0;
    write_in_steps(
        &mut result,
        &mut line_id_map,
        &mut line,
        &test_case.steps,
        0,
    );
    (result, line_id_map)
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
    let mut file_name_lines_id_map: HashMap<String, HashMap<Side, HashMap<usize, i64>>> =
        Default::default();

    for test_case in left_loners.into_iter() {
        let case_with_steps = get_case(&db, test_case.id).await;
        let (content, line_id_map) = case_to_string(&case_with_steps);
        file_name_lines_id_map.insert(test_case.name.clone(), hash_map! {Side::Left: line_id_map});

        let mut hunk = String::default();
        writeln!(&mut hunk, "--- {}", test_case.name).unwrap();
        writeln!(&mut hunk, "+++ {}", test_case.name).unwrap();
        writeln!(&mut hunk, "@@ @@").unwrap();
        for line in content.lines() {
            writeln!(&mut hunk, "- {line}").unwrap();
        }

        diffs += hunk.to_string().as_str();
    }

    for test_case in right_loners.into_iter() {
        let case_with_steps = get_case(&db, test_case.id).await;
        let (content, line_id_map) = case_to_string(&case_with_steps);
        file_name_lines_id_map.insert(test_case.name.clone(), hash_map! {Side::Right: line_id_map});

        let mut hunk = String::default();
        writeln!(&mut hunk, "--- {}", test_case.name).unwrap();
        writeln!(&mut hunk, "+++ {}", test_case.name).unwrap();
        writeln!(&mut hunk, "@@ @@").unwrap();
        for line in content.lines() {
            writeln!(&mut hunk, "+ {line}").unwrap();
        }

        diffs += hunk.to_string().as_str();
    }

    for (left_test_case, right_test_case) in matches.into_iter() {
        let mut hunk = String::default();

        let l_case_with_steps = get_case(&db, left_test_case.id).await;
        let r_case_with_steps = get_case(&db, right_test_case.id).await;

        let (l, l_line_id_map) = case_to_string(&l_case_with_steps);
        let (r, r_line_id_map) = case_to_string(&r_case_with_steps);

        file_name_lines_id_map.insert(
            left_test_case.name.clone(),
            hash_map! {
                Side::Left: l_line_id_map,
                Side::Right: r_line_id_map
            },
        );

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

    let raw_templates = r#"{
        "tag-file-changed": '<span class="d2h-tag d2h-changed d2h-changed-tag">COOL</span>',
        "generic-line": `
            <tr>
                <td class="{{lineClass}} {{type}}">
                {{{lineNumber}}}
                </td>
                <td class="{{type}}">
                    <div class="{{contentClass}}">
                    {{#prefix}}
                        <span class="d2h-code-line-prefix">{{{prefix}}}</span>
                    {{/prefix}}
                    {{^prefix}}
                        <span class="d2h-code-line-prefix">&nbsp;</span>
                    {{/prefix}}
                    {{#content}}
                        <span class="d2h-code-line-ctn hover:underline hover:cursor-pointer" onclick="handle_line_click(this, {{{lineNumber}}})">{{{content}}}</span>
                    {{/content}}
                    {{^content}}
                        <span class="d2h-code-line-ctn"><br></span>
                    {{/content}}
                    </div>
                </td>
            </tr>
        `
    }"#
    .to_string();

    Html(
        TemplateInstance {
            diff: diffs,
            raw_templates,
            map: serde_json::to_string(&file_name_lines_id_map).unwrap(),
        }
        .render()
        .unwrap(),
    )
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/:left_run_id/:right_run_id", get(html))
        .with_state(db)
}
