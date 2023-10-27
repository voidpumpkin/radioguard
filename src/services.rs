use std::cmp::Ordering;
use std::io::Cursor;

use base64::Engine;
use image::DynamicImage;
use image::ImageFormat;
use image::ImageOutputFormat;

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

pub async fn compare_steps(left_step_id: &str, right_step_id: &str) -> (bool, String) {
    let l_img = data_uri_to_dyn_img(left_step_id);
    let r_img = data_uri_to_dyn_img(right_step_id);

    let (f, out_img) = img_diff::subtract_image(&l_img, &r_img);
    let contains_changes = f.total_cmp(&0.0_f64) == Ordering::Greater;

    // We will write the image data to a byte vector in PNG format.
    let mut bytes: Vec<u8> = Vec::new();
    out_img
        .write_to(&mut bytes, ImageOutputFormat::PNG)
        .unwrap();

    // Now, we encode these bytes into a base64 string.
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);
    (
        contains_changes,
        format!("data:@file/png;base64,{base64_string}"),
    )
}
