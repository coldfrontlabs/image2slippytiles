use crate::cli::Cli;
use image::DynamicImage;


pub fn thumbnailfromtiles(
    args: Cli,
) {
    let min_tile_path = format!("{}/0/0/0.{}", args.output, args.format);
    let min_tile = image::open(&min_tile_path).unwrap();
    let min_size = u32::min(min_tile.width(), min_tile.height());

    let canary_tile_path = format!("{}/1/0/0.{}", args.output, args.format);
    let canary_tile = image::open(&canary_tile_path).unwrap();
    let tile_size = canary_tile.width();

    let scale = args.thumbnailsize as f32 / min_size as f32;
    let tiles_needed = (args.thumbnailsize as f32 / tile_size as f32).ceil() as u32;


}