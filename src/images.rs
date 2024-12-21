use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageResult, RgbaImage};
use reqwest::get;
use rustemon::client::RustemonClient;
use rustemon::error::Error;
use rustemon::model::pokemon::Pokemon;

pub(crate) async fn get_file(name: &str, rustemon_client: &RustemonClient) -> String {
    let path: String = "static/".to_owned() + name + ".png";
    // let path = ;
    if !Path::new(&path).exists() {
        let mon = match rustemon::pokemon::pokemon::get_by_name(name, &rustemon_client).await {
            Ok(mon) => mon,
            Err(_) => {return "/static/MissingNo2.png".to_owned()}
        };

        let url = mon.sprites.front_default.unwrap();
        // Download the image
        let response = match get(url).await {
            Ok(response) => response,
            Err(_) => {return "/static/MissingNo2.png".to_owned()}
        };

        let bytes = response.bytes().await.unwrap();
        let mut img = match image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
            Ok(mut img) => img,
            Err(_) => {return "/static/MissingNo2.png".to_owned()}
        };
        // Crop the image to remove transparent margins
        let cropped_img = crop_to_remove_transparent_margins(&mut img);

        // Save the cropped image
        let _ = cropped_img.save(&path);
    }
    path
}

fn crop_to_remove_transparent_margins(img: &mut DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            if img.get_pixel(x, y)[3] != 0 {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    let cropped_img = img.crop(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
    DynamicImage::ImageRgba8(RgbaImage::from(cropped_img))
}