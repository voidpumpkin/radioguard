use std::cmp::Ordering;
use std::io::Cursor;

use askama::Template;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use base64::Engine;
use image::DynamicImage;
use image::ImageFormat;
use image::ImageOutputFormat;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::db::get_step_data_uri;
use crate::models::side::Side;

#[derive(Template)]
#[template(path = "frontend/pages/steps.jinja", escape = "none")]
struct TemplateInstance {
    list: Vec<ListItem>,
}

struct ListItem {
    unique_id: String,
    data_uri: String,
    cta: String,
    img_css: String,
}

fn data_uri_to_dyn_img(data_uri: &str) -> DynamicImage {
    // Split the URI to separate the metadata from the actual encoded data
    let split: Vec<&str> = data_uri.split(',').collect();
    if split.len() != 2 {
        panic!("Invalid data URI");
    }
    let data = split[1];

    // Decode the base64 portion
    let decoded = &base64::engine::general_purpose::STANDARD
        .decode(data)
        .expect("Decoding base64 failed");

    // Create a cursor for the byte slice, because the image crate needs a reader.
    let cursor = Cursor::new(decoded);
    // Use the reader to load the image and convert it into a DynamicImage.
    // It automatically detects the format (PNG in this case).
    image::load(cursor, ImageFormat::PNG).unwrap()
}

async fn html_single(State(db): State<Pool<Sqlite>>, Path(step_id): Path<i64>) -> Html<String> {
    let data_uri = get_step_data_uri(step_id, &db).await;

    Html(
        TemplateInstance {
            list: vec![ListItem {
                unique_id: "single".to_string(),
                data_uri,
                cta: "🖼️".to_string(),
                img_css: "".to_string(),
            }],
        }
        .render()
        .unwrap(),
    )
}

async fn html_diff(
    State(db): State<Pool<Sqlite>>,
    Path((left_step_id, right_step_id)): Path<(i64, i64)>,
) -> Html<String> {
    let left_data_uri = get_step_data_uri(left_step_id, &db).await;
    let right_data_uri = get_step_data_uri(right_step_id, &db).await;

    let l_img = data_uri_to_dyn_img(left_data_uri.as_str());
    let r_img = data_uri_to_dyn_img(right_data_uri.as_str());

    let (f, out_img) = img_diff::subtract_image(&l_img, &r_img);
    let contains_changes = f.total_cmp(&0.0_f64) == Ordering::Greater;

    // We will write the image data to a byte vector in PNG format.
    let mut bytes: Vec<u8> = Vec::new();
    out_img
        .write_to(&mut bytes, ImageOutputFormat::PNG)
        .unwrap();

    // Now, we encode these bytes into a base64 string.
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);

    let mut list = vec![];
    list.push(ListItem {
        unique_id: Side::Left.to_string(),
        data_uri: left_data_uri,
        cta: "👈".to_string(),
        img_css: "".to_string(),
    });
    if contains_changes {
        list.push(ListItem {
            unique_id: "diff".to_string(),
            data_uri: format!("data:@file/png;base64,{base64_string}"),
            cta: "🤝".to_string(),
            img_css: "invert".to_string(),
        });
    }
    list.push(ListItem {
        unique_id: Side::Right.to_string(),
        data_uri: right_data_uri,
        cta: "👉".to_string(),
        img_css: "".to_string(),
    });
    Html(TemplateInstance { list }.render().unwrap())
}

pub fn router(db: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/:step_id", get(html_single))
        .route("/:left_step_id/:right_step_id", get(html_diff))
        .with_state(db)
}
