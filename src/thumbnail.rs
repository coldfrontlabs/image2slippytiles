use crate::cli::Cli;
use image::{DynamicImage, GenericImageView};
use std::path::Path;

pub fn thumbnailfromtiles(args: &Cli, dimension: Option<(u32, u32)>) {
    let min_tile_path_res = tile_path(
        &args.output,
        0,
        0,
        0,
        &args.format,
        args.thumbnailfromzoomifytiles,
    );
    if min_tile_path_res.is_err() {
        println!("{}", min_tile_path_res.unwrap_err());
        return;
    }
    let min_tile_path = min_tile_path_res.unwrap();
    let min_tile = image::open(&min_tile_path).unwrap();
    let min_size = real_min_size(&min_tile);

    let mut min_available_zoom = 12;
    let mut max_available_zoom = 0;
    for i in 0..12 {
        if tile_path(
            &args.output,
            0,
            0,
            i,
            &args.format,
            args.thumbnailfromzoomifytiles,
        )
        .is_ok()
        {
            if i < min_available_zoom {
                min_available_zoom = i;
            }
            if i > max_available_zoom {
                max_available_zoom = i;
            }
        }
    }

    let canary_tile_path = tile_path(
        &args.output,
        0,
        0,
        max_available_zoom,
        &args.format,
        args.thumbnailfromzoomifytiles,
    )
    .unwrap();

    let canary_tile = image::open(&canary_tile_path).unwrap();
    let tile_size = canary_tile.width();

    let scale_x = min_size.0 as f32 / tile_size as f32;
    let scale_y = min_size.1 as f32 / tile_size as f32;
    let tiles_needed = (args.thumbnailsize as f32 / tile_size as f32).ceil() as u32;

    let mut known_final_size = false;
    let mut final_size = [0, 0];
    let mut zoom = (tiles_needed as f32).ln().ceil() as u32 + 1;

    if let Some((width, height)) = dimension {
        let width_ratio = width as f32 / (u32::pow(2, max_available_zoom) * tile_size) as f32;
        let height_ratio = height as f32 / (u32::pow(2, max_available_zoom) * tile_size) as f32;
        for i in min_available_zoom..max_available_zoom {
            if args.thumbnailsize
                < (width_ratio * u32::pow(i, 2) as f32 * tile_size as f32).ceil() as u32
                && args.thumbnailsize
                    < (height_ratio * u32::pow(i, 2) as f32 * tile_size as f32).ceil() as u32
            {
                zoom = i;
                break;
            }
        }

        known_final_size = true;
        final_size[0] = (width_ratio * u32::pow(2, zoom) as f32 * tile_size as f32).floor() as u32;
        final_size[1] = (height_ratio * u32::pow(2, zoom) as f32 * tile_size as f32).floor() as u32;
    }

    if zoom > max_available_zoom {
        zoom = max_available_zoom;
    }
    if zoom < min_available_zoom {
        zoom = min_available_zoom;
    }

    if args.verbose {
        println!(
            "Thumbnail size: {},{} - from zoom {}",
            final_size[0], final_size[1], zoom
        );
    }

    let mut buffer = DynamicImage::new_rgba8(
        ((u32::pow(2, zoom) * tile_size) as f32 * scale_x).floor() as u32,
        ((u32::pow(2, zoom) * tile_size) as f32 * scale_y).floor() as u32,
    );

    for x in 0..u32::pow(2, zoom) {
        for y in 0..u32::pow(2, zoom) {
            let tile_path_res = tile_path(
                &args.output,
                x,
                y,
                zoom,
                &args.format,
                args.thumbnailfromzoomifytiles,
            );
            if tile_path_res.is_err() {
                break;
            }
            let tile_path = tile_path_res.unwrap();
            let tile = image::open(&tile_path).unwrap();
            let x_offset = x * tile_size;
            let y_offset = y * tile_size;
            image::imageops::overlay(&mut buffer, &tile, x_offset as i64, y_offset as i64);

            if x == 0 && !known_final_size {
                final_size[1] += tile.height();
            }
            if y == 0 && !known_final_size {
                final_size[0] += tile.width();
            }
        }
    }

    buffer = buffer.crop_imm(0, 0, final_size[0], final_size[1]);
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

fn tile_path(
    output: &String,
    x: u32,
    y: u32,
    z: u32,
    format: &String,
    zoomify: bool,
) -> Result<String, String> {
    if zoomify {
        for g in 0..32 {
            let file = format!("{}/TileGroup{}/{}-{}-{}.{}", output, g, z, x, y, format);
            if Path::new(&file).exists() {
                return Ok(file);
            }
        }
        Err(format!(
            "File not found for {}/TileGroupX/{}-{}-{}.{}",
            output, z, x, y, format
        ))
    } else {
        let path = format!("{}/{}/{}/{}.{}", output, z, x, y, format);
        if Path::new(&path).exists() {
            Ok(path)
        } else {
            Err(format!("File not found for {}", path))
        }
    }
}
