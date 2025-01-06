use crate::cli::Cli;
use image::{DynamicImage, GenericImageView};
use std::path::Path;

pub fn thumbnailfromtiles(args: Cli) {
    let min_tile_path = format!("{}/0/0/0.{}", args.output, args.format);
    let min_tile = image::open(&min_tile_path).unwrap();
    let min_size = real_min_size(&min_tile);

    let canary_tile_path = format!("{}/1/0/0.{}", args.output, args.format);
    let canary_tile = image::open(&canary_tile_path).unwrap();
    let tile_size = canary_tile.width();

    let scale_x = min_size.0 as f32 / tile_size as f32;
    let scale_y = min_size.1 as f32 / tile_size as f32;
    let tiles_needed = (args.thumbnailsize as f32 / tile_size as f32).ceil() as u32;

    let zoom = (tiles_needed as f32).ln().ceil() as u32 + 1;

    let mut buffer = DynamicImage::new_rgba8(
        ((zoom.pow(2) * tile_size) as f32 * scale_x).floor() as u32,
        ((zoom.pow(2) * tile_size) as f32 * scale_y).floor() as u32,
    );

    for x in 0..zoom.pow(2) {
        for y in 0..zoom.pow(2) {
            let tile_path = format!("{}/{}/{}/{}.{}", args.output, zoom, x, y, args.format);
            if !Path::new(&tile_path).exists() {
                break;
            }
            let tile = image::open(&tile_path).unwrap();
            let x_offset = x * tile_size;
            let y_offset = y * tile_size;
            image::imageops::overlay(&mut buffer, &tile, x_offset as i64, y_offset as i64);
        }
    }

    let rgb = buffer.to_rgb8();
    rgb.save_with_format(
        format!("{}/thumbnail.jpg", args.output),
        image::ImageFormat::Jpeg,
    )
    .unwrap();
}

pub fn real_min_size(image: &DynamicImage) -> (u32, u32) {
    let mut width = image.width();
    let mut height = image.height();

    for x in 0..width {
        let pixel = image.get_pixel(x, 0);
        if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 0 {
            width = x - 1;
            break;
        }
    }

    for y in 0..height {
        let pixel = image.get_pixel(0, y);
        if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 0 {
            height = y - 1;
            break;
        }
    }
    (width, height)
}
