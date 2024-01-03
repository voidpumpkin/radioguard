use std::cmp::max;
use std::cmp::Ordering;
use std::io::Cursor;

use anyhow::Result;
use base64::Engine;
use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use image::ImageFormat;
use image::ImageOutputFormat;

fn data_uri_to_dyn_img(data_uri: &str) -> Result<DynamicImage> {
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
    Ok(image::load(cursor, ImageFormat::PNG)?)
}

pub async fn compare_steps(
    left_step_id: &str,
    right_step_id: &str,
    ignore_ranges: &[((u32, u32), (u32, u32))],
) -> Result<(bool, String)> {
    let l_img = data_uri_to_dyn_img(left_step_id)?;
    let r_img = data_uri_to_dyn_img(right_step_id)?;

    let (f, out_img) = subtract_image(&l_img, &r_img, ignore_ranges);
    let contains_changes = f.total_cmp(&0.0_f64) == Ordering::Greater;

    // We will write the image data to a byte vector in PNG format.
    let mut bytes: Vec<u8> = Vec::new();
    out_img.write_to(&mut bytes, ImageOutputFormat::PNG)?;

    // Now, we encode these bytes into a base64 string.
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok((
        contains_changes,
        format!("data:@file/png;base64,{base64_string}"),
    ))
}

/// Originally taken from img_diff library
pub fn subtract_image(
    a: &DynamicImage,
    b: &DynamicImage,
    ignore_ranges: &[((u32, u32), (u32, u32))],
) -> (f64, DynamicImage) {
    let (x_dim, y_dim) = a.dimensions();
    let mut diff_image = DynamicImage::new_rgba8(x_dim, y_dim);
    let mut max_value: f64 = 0.0;
    let mut current_value: f64 = 0.0;
    for ((x, y, pixel_a), (_, _, pixel_b)) in a.pixels().zip(b.pixels()) {
        for ((x1, y1), (x2, y2)) in ignore_ranges {
            if (*x1..=*x2).contains(&x) && (*y1..=*y2).contains(&y) {
                diff_image.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
                continue;
            }
        }

        // TODO(miguelmendes): find a way to avoid groups of 4 algorithm
        max_value += f64::from(max(pixel_a[0], pixel_b[0]));
        max_value += f64::from(max(pixel_a[1], pixel_b[1]));
        max_value += f64::from(max(pixel_a[2], pixel_b[2]));
        max_value += f64::from(max(pixel_a[3], pixel_b[3]));
        let r = subtract_and_prevent_overflow(pixel_a[0], pixel_b[0]);
        let g = subtract_and_prevent_overflow(pixel_a[1], pixel_b[1]);
        let b = subtract_and_prevent_overflow(pixel_a[2], pixel_b[2]);
        let a = subtract_and_prevent_overflow(pixel_a[3], pixel_b[3]);
        current_value += f64::from(r);
        current_value += f64::from(g);
        current_value += f64::from(b);
        current_value += f64::from(a);
        diff_image.put_pixel(x, y, image::Rgba([255 - r, 255 - g, 255 - b, 255 - a]));
    }
    (((current_value * 100.0) / max_value), diff_image)
}

/// taken from img_diff
fn subtract_and_prevent_overflow(a: u8, b: u8) -> u8 {
    if a > b {
        a - b
    } else {
        b - a
    }
}
