use crate::chunkable::*;
use crate::cli::Cli;
use crate::globals::PEAK_ALLOC;
use crate::metadata::*;
use crate::thumbnail::*;
use futures::future::join_all;
use hex_color::HexColor;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use std::fs;
use std::time::Instant;

#[derive(Debug)]

pub struct Tile {
    tile_id: (u32, u32, u32),
    size: u32,
    chunk_bounds: (u32, u32, u32, u32),
}

pub struct ImageTile {
    pub tile_id: (u32, u32, u32),
    pub image: DynamicImage,
}

impl ImageTile {
    pub fn save(&self, path: &str, format: &str) {
        fs::create_dir(format!("{}/{}", path, self.tile_id.0)).unwrap_or_default();
        fs::create_dir(format!("{}/{}/{}", path, self.tile_id.0, self.tile_id.1))
            .unwrap_or_default();
        let name: String = format!(
            "{}/{}/{}/{}.{}",
            path, self.tile_id.0, self.tile_id.1, self.tile_id.2, format
        );
        let image_format = match format {
            "png" => image::ImageFormat::Png,
            "jpg" => image::ImageFormat::Jpeg,
            "webp" => image::ImageFormat::WebP,
            _ => image::ImageFormat::Png,
        };
        if image_format == image::ImageFormat::Jpeg {
            let rgb = self.image.to_rgb8();
            rgb.save_with_format(name, image_format).unwrap();
        } else {
            self.image.save_with_format(name, image_format).unwrap();
        }
    }
}

pub async fn tilechunker(
    args: Cli,
    final_tile_size: u32,
    chunk_zoom: u32,
    source: impl ChunkSource,
    start_time: Instant,
) -> Result<TileMetadata, String> {
    if args.debug {
        println!("Generating tiles from image ...")
    }

    let image_metadata = source.get_image_metadata();
    let default_hexcolour = HexColor::parse_rgba(args.colour.as_str()).unwrap();
    let default_colour = [
        default_hexcolour.r,
        default_hexcolour.g,
        default_hexcolour.b,
        default_hexcolour.a,
    ];

    let max = std::cmp::max(image_metadata.width, image_metadata.height);
    let scale = (max as f32 / final_tile_size as f32).ceil() as u32;
    let max_zoom = (scale as f32).log2().ceil() as u32;

    let chunk_size = u32::pow(2, chunk_zoom) * final_tile_size;

    let width_chunks = (image_metadata.width as f32 / chunk_size as f32).ceil() as u32;
    let height_chunks = (image_metadata.height as f32 / chunk_size as f32).ceil() as u32;

    let mut max_chunk_zoom = 0;
    if chunk_zoom < max_zoom {
        max_chunk_zoom = max_zoom - chunk_zoom;
    }

    if args.verbose {
        println!(
            "Dividing {}x{} images into {}x{} chunks of {}",
            image_metadata.width, image_metadata.height, width_chunks, height_chunks, chunk_size
        );
        println!(
            "Max zoom is {}, can generate up to zoom level {} from each chunk",
            max_zoom, max_chunk_zoom
        );
    }

    if args.debug {
        println!(
            "Max dimension of source image: {}\nFinal tile size: {}\nChunk Zoom: {}\nScale: {}\nMax zoom: {}\nOutput path: {}\nChunk Size: {}\nWidth Chunks: {}\nHeight Chunks: {}\nMax Chunks zoom:{}",
            max,
            final_tile_size,
            chunk_zoom,
            scale,
            max_zoom,
            &args.output,
            chunk_size,
            width_chunks,
            height_chunks,
            max_chunk_zoom
        );
    }

    /*
    let mut total_tiles = 1;
    for z in 1..max_zoom+1 {
        let tile_size = u32::pow(2, max_zoom - z) * final_tile_size;
        let width_tiles_at_zoom = (width_chunks * chunk_size) / tile_size;
        let height_tiles_at_zoom = (height_chunks * chunk_size) / tile_size;
        total_tiles += width_tiles_at_zoom * height_tiles_at_zoom;
    }
    print!("Total tiles needed: {}", total_tiles);
    */

    fs::create_dir(&args.output).unwrap_or_default();

    /*
    let thumbnail_scale = args.thumbnailsize as f32
        / std::cmp::min(image_metadata.width, image_metadata.height) as f32;
    let thumbnail_chunk_size = (chunk_size as f32 * thumbnail_scale).ceil() as u32;


    let thumbnail_size = [
        (width_chunks * thumbnail_chunk_size) as u32,
        (height_chunks * thumbnail_chunk_size) as u32,
    ];
    */
    /*
        let mut thumbnail: DynamicImage = image::DynamicImage::from(image::ImageBuffer::from_pixel(
            thumbnail_size[0],
            thumbnail_size[1],
            image::Rgba([0_u8, 0_u8, 0_u8, 0_u8]),
        ));

        let mut thumbnail_actual = [0, 0];
    */
    //let mut tiles_processed = 0;

    let mut handles = Vec::new();
    for x in 0..width_chunks {
        for y in 0..height_chunks {
            let chunk_id = (x, y);

            if args.verbose {
                println!("Chunk ID: {:?}", chunk_id);
            }

            let mut chunk = source.get_chunk(chunk_id, chunk_size, chunk_size);
            /*
                        if args.thumbnail {
                            let thumbnail_chunk: DynamicImage = chunk.resize(
                                (thumbnail_chunk_size as f32 * (chunk.width() as f32 / chunk_size as f32))
                                    .ceil() as u32,
                                (thumbnail_chunk_size as f32 * (chunk.height() as f32 / chunk_size as f32))
                                    .ceil() as u32,
                                FilterType::Lanczos3,
                            );

                            if y == 0 {
                                thumbnail_actual[0] += thumbnail_chunk.width();
                            }

                            if x == 0 {
                                thumbnail_actual[1] += thumbnail_chunk.height();
                            }

                            thumbnail_chunk.pixels().for_each(|(i, j, pixel)| {
                                let placement = [
                                    i as i32 + (x * thumbnail_chunk_size) as i32,
                                    j as i32 + (y * thumbnail_chunk_size) as i32,
                                ];

                                if placement[0] > 0
                                    && placement[0] < thumbnail_size[0] as i32
                                    && placement[1] > 0
                                    && placement[1] < thumbnail_size[1] as i32
                                {
                                    thumbnail.put_pixel(placement[0] as u32, placement[1] as u32, pixel);
                                }
                            });
                        }
            */
            if chunk.width() != chunk_size || chunk.height() != chunk_size {
                if args.debug {
                    println!("Generate a partial chunk");
                }
                let mut chunk_with_background =
                    image::DynamicImage::from(image::ImageBuffer::from_pixel(
                        chunk_size,
                        chunk_size,
                        image::Rgba(default_colour),
                    ));
                chunk
                    .pixels()
                    .for_each(|(x, y, pixel)| chunk_with_background.put_pixel(x, y, pixel));
                chunk = chunk_with_background;
            }

            if args.debug {
                println!("Generating tiles from chunk");
            }

            // process chunk.
            handles.push(tokio::spawn(process_chunk(
                chunk,
                x,
                y,
                max_zoom,
                max_chunk_zoom,
                chunk_size,
                final_tile_size,
                image_metadata.width,
                image_metadata.height,
                args.output.clone(),
                args.format.clone(),
                args.verbose,
                args.debug,
            )));
            // Process chunk end.
            timeout(args.timeout, start_time)?;
            if args.debug {
                println!("Done tile generation for chunk.")
            }
        }
    }
    join_all(handles).await;

    if args.debug {
        println!("Done processing all chunks.");
        println!(
            "Generating lower zoom tiles from parent tiles - From {} to {}",
            args.zoom, max_chunk_zoom
        );
    }

    for z in (args.zoom..max_chunk_zoom).rev() {
        let tiles_at_zoom = u32::pow(2, z);

        for x in 0..tiles_at_zoom {
            for y in 0..tiles_at_zoom {
                let tile_id = (z, x, y);
                let tile = Tile {
                    tile_id,
                    size: 0,
                    chunk_bounds: (0, 0, 0, 0),
                };

                if args.debug {
                    println!("Compiling tile {:?}", tile_id);
                }

                timeout(args.timeout, start_time)?;

                if let Some(tileimage) = generate_compiled_tile(
                    tile,
                    final_tile_size,
                    &args.output,
                    &args.format,
                    &default_colour,
                ) {
                    tileimage.save(&args.output, args.format.as_str());
                }
            }
        }
    }

    if args.debug {
        println!("Done processing all lower zoom tiles.");
        println!("Generating thumbnail.")
    }

    if args.thumbnail {
        /*
        thumbnail
            .crop(0, 0, thumbnail_actual[0], thumbnail_actual[1])
            .to_rgb8()
            .save_with_format(format!("{}/thumbnail.jpg", path), image::ImageFormat::Jpeg)
            .unwrap();
        */
        thumbnailfromtiles(&args, Some((image_metadata.width, image_metadata.height)));
    }

    Ok(TileMetadata {
        min_zoom: args.zoom,
        max_zoom,
        bounds: [
            0.0,
            0.0,
            -1.0 * image_metadata.height as f32 / u32::pow(2, max_zoom) as f32,
            image_metadata.width as f32 / u32::pow(2, max_zoom) as f32,
        ],
        image_type: args.format,
        peak_memory: PEAK_ALLOC.peak_usage_as_mb(),
        duration: start_time.elapsed().as_secs_f32(),
        image_metadata: source.get_image_metadata(),
        slide_metadata: source.get_slide_metadata(),
    })
}

/**
 * Generate a full tile from a set of source tiles.
 */
pub fn generate_full_tile(source: &DynamicImage, tile: Tile, tile_size: u32) -> ImageTile {
    let mut tile_img = source.crop_imm(
        tile.chunk_bounds.0,
        tile.chunk_bounds.1,
        tile.size,
        tile.size,
    );
    if tile.size != tile_size {
        tile_img = tile_img.resize(tile_size, tile_size, FilterType::Nearest);
    }
    ImageTile {
        image: tile_img,
        tile_id: tile.tile_id,
    }
}

/**
 * Generate a compiled tile from a set of source tiles.
 */
pub fn generate_compiled_tile(
    tile: Tile,
    tile_size: u32,
    path: &String,
    format: &String,
    colour: &[u8; 4],
) -> Option<ImageTile> {
    let source_zoom = tile.tile_id.0 + 1;
    let source_tiles = [
        (tile.tile_id.1 * 2, tile.tile_id.2 * 2, 0, 0),
        (tile.tile_id.1 * 2 + 1, tile.tile_id.2 * 2, 1, 0),
        (tile.tile_id.1 * 2, tile.tile_id.2 * 2 + 1, 0, 1),
        (tile.tile_id.1 * 2 + 1, tile.tile_id.2 * 2 + 1, 1, 1),
    ];

    let mut buffer = image::DynamicImage::from(image::ImageBuffer::from_pixel(
        tile_size * 2,
        tile_size * 2,
        image::Rgba(*colour),
    ));

    let mut oops_all_blanks = true;

    for source_tile in source_tiles {
        let source_file = format!(
            "{}/{}/{}/{}.{}",
            path, source_zoom, source_tile.0, source_tile.1, format
        );
        if let Ok(tile_img) = image::open(source_file) {
            oops_all_blanks = false;
            buffer
                .copy_from(
                    &tile_img,
                    source_tile.2 * tile_size,
                    source_tile.3 * tile_size,
                )
                .unwrap();
        }
    }
    if oops_all_blanks {
        return None;
    }

    buffer = buffer.resize(tile_size, tile_size, FilterType::Lanczos3);

    Some(ImageTile {
        image: buffer,
        tile_id: tile.tile_id,
    })
}

pub fn timeout(timeout: u64, start_time: Instant) -> Result<(), String> {
    if timeout == 0 {
        return Ok(());
    }

    if start_time.elapsed().as_secs() > timeout {
        return Err(format!(
            "Timeout exceeded: Took {}, allowed: {}",
            start_time.elapsed().as_secs_f32(),
            timeout
        )
        .to_string());
    }
    Ok(())
}

async fn process_chunk(
    chunk: DynamicImage,
    x: u32,
    y: u32,
    max_zoom: u32,
    max_chunk_zoom: u32,
    chunk_size: u32,
    final_tile_size: u32,
    image_metadata_width: u32,
    image_metadata_height: u32,
    output: String,
    format: String,
    verbose: bool,
    debug: bool,
) -> u32 {
    let mut tiles_processed = 0;
    for z in max_chunk_zoom..max_zoom + 1 {
        let tile_size = u32::pow(2, max_zoom - z) * final_tile_size;

        if verbose {
            println!(
                "Zoom level: {}, pre-resize tile size: {}, from chunk size: {}",
                z, tile_size, chunk_size
            );
        }

        let width_tiles_at_zoom = chunk_size / tile_size as u32;
        let height_tiles_at_zoom = chunk_size / tile_size as u32;

        for tilex in 0..width_tiles_at_zoom {
            for tiley in 0..height_tiles_at_zoom {
                let tile_id = (
                    z,
                    tilex + (x * width_tiles_at_zoom),
                    tiley + (y * height_tiles_at_zoom),
                );
                let tile_bounds_in_chunk = (
                    tilex * tile_size,
                    tiley * tile_size,
                    tilex * tile_size + tile_size,
                    tiley * tile_size + tile_size,
                );
                let tile_bounds_in_image = (
                    tile_bounds_in_chunk.0 + (x * chunk_size),
                    tile_bounds_in_chunk.1 + (y * chunk_size),
                    tile_bounds_in_chunk.2 + (x * chunk_size),
                    tile_bounds_in_chunk.3 + (y * chunk_size),
                );

                if debug {
                    println!(
                        "Tile ID: {:?}, Bounds in chunk: {:?}, Bounds in image: {:?}",
                        tile_id, tile_bounds_in_chunk, tile_bounds_in_image
                    );
                }
                let tile = Tile {
                    tile_id,
                    size: tile_size,
                    chunk_bounds: tile_bounds_in_chunk,
                };

                if tile_bounds_in_image.0 > image_metadata_width
                    || tile_bounds_in_image.1 > image_metadata_height
                {
                    if debug {
                        println!("Tile out of bounds");
                    }
                    break;
                } else {
                    if debug {
                        println!("Full tile");
                    }
                    generate_full_tile(&chunk, tile, final_tile_size).save(&output, &format);
                    tiles_processed += 1;
                }
            }
        }
    }
    return tiles_processed;
}
