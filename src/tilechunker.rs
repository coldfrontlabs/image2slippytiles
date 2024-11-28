use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use std::fs;
use crate::metadata::*;
use crate::globals::PEAK_ALLOC;
use crate::cli::Cli;
use crate::chunkable::*;

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
        fs::create_dir(format!("{}/{}/{}", path, self.tile_id.0, self.tile_id.1)).unwrap_or_default();
        let name: String = format!("{}/{}/{}/{}.{}", path, self.tile_id.0, self.tile_id.1, self.tile_id.2, format);
        let image_format = match format {
            "png" => image::ImageFormat::Png,
            "jpg" => image::ImageFormat::Jpeg,
            "webp" => image::ImageFormat::WebP,
            _ => image::ImageFormat::Png,
        };
        self.image.save_with_format(name, image_format)
            .unwrap();
    }
}

pub fn tilechunker(args: Cli, final_tile_size: u32, chunk_zoom: u32, source: impl ChunkSource) -> TileMetadata {
    let image_metadata = source.get_image_metadata();

    let max = std::cmp::max(image_metadata.width, image_metadata.height);
    let scale = (max as f32 / 256.0).ceil() as u32;
    let max_zoom = (scale as f32).log2().ceil() as u32;
    let path = args.output.unwrap_or("./tiles".to_string());

    let chunk_size = u32::pow(2, chunk_zoom) * final_tile_size;

    let width_chunks = (image_metadata.width as f32 / chunk_size as f32).ceil() as u32;
    let height_chunks = (image_metadata.height as f32 / chunk_size as f32).ceil() as u32;

    let max_chunk_zoom = max_zoom - chunk_zoom;

    if args.verbose {
        println!("Dividing {}x{} images into {}x{} chunks of {}", image_metadata.width, image_metadata.height, width_chunks, height_chunks, chunk_size);
        println!("Max zoom is {}, can generate up to zoom level {} from each chunk", max_zoom, max_chunk_zoom);
    }

    fs::create_dir(format!("{}", path)).unwrap_or_default();

    for x in 0..width_chunks {
        for y in 0..height_chunks {
            let chunk_id = (x, y);

            if args.verbose {
                println!("Chunk ID: {:?}", chunk_id);
            }
    
            let chunk = source.get_chunk(chunk_id, chunk_size, chunk_size);
            for z in max_chunk_zoom..max_zoom+1 {
                let tile_size = u32::pow(2, max_zoom - z) * 256;

                if args.verbose {
                    println!("Zoom level: {}, pre-resize tile size: {}", z, tile_size);
                }
                let mut count = 0;

                let width_tiles_at_zoom = chunk_size / tile_size as u32;
                let height_tiles_at_zoom = chunk_size / tile_size as u32;

                for tilex in 0..width_tiles_at_zoom {
                    for tiley in 0..height_tiles_at_zoom {
                        let tile_id = (z, tilex + (x * width_tiles_at_zoom), tiley + (y * height_tiles_at_zoom));
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

                        if args.debug {
                            println!("Tile ID: {:?}, Bounds in chunk: {:?}, Bounds in image: {:?}", tile_id, tile_bounds_in_chunk, tile_bounds_in_image);
                        }
                        let tile = Tile {
                            tile_id: tile_id,
                            size: tile_size,
                            chunk_bounds: tile_bounds_in_chunk,
                        };

                        if tile_bounds_in_image.0 > image_metadata.width || tile_bounds_in_image.1 > image_metadata.height {
                            if args.debug {
                                println!("Tile out of bounds");
                            }
                            break;
                        }
                        else if tile_bounds_in_image.2 > image_metadata.width || tile_bounds_in_image.3 > image_metadata.height {
                            if args.debug {
                                println!("Partial tile");
                            }
                            generate_partial_tile(&chunk, tile, final_tile_size).save(&path, args.format.as_str());
                        }
                        else {
                            if args.debug {
                                println!("Full tile");
                            }
                            generate_full_tile(&chunk, tile, final_tile_size).save(&path, args.format.as_str());
                        }

                        count = count + 1;
                    }
                }
                if args.verbose {
                    println!("{} tiles at zoom level {} in chunk {}, {}\n", count, z, x, y);
                }
            }
        }
    }

    for z in (args.zoom..max_chunk_zoom).rev() {
        let tiles_at_zoom = u32::pow(2, z);
        for x in 0..tiles_at_zoom {
            for y in 0..tiles_at_zoom {
                let tile_id = (z, x, y);
                let tile = Tile {
                    tile_id: tile_id,
                    size: 0,
                    chunk_bounds: (0, 0, 0, 0),
                };

                if args.debug {
                    println!("Compiling tile {:?}", tile_id);
                }
                generate_compiled_tile(tile, final_tile_size, &path, &args.format).save(&path, args.format.as_str());
            }
        }
    }

    TileMetadata {
        min_zoom: args.zoom,
        max_zoom: max_zoom,
        bounds: [
            0.0,
            0.0,
            -1.0 * image_metadata.height as f32 / u32::pow( 2, max_zoom) as f32,
            image_metadata.width as f32 /u32::pow( 2, max_zoom) as f32,
        ],
        image_type: args.format,
        peak_memory: PEAK_ALLOC.peak_usage_as_mb(),
        image_metadata: source.get_image_metadata(),
        slide_metadata: source.get_slide_metadata(),
    }
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
    return ImageTile {
        image: tile_img,
        tile_id: tile.tile_id,
    };
}

/**
 * Generate a partial tile from a set of source tiles.
 */
pub fn generate_partial_tile(source: &DynamicImage, tile: Tile, tile_size: u32) -> ImageTile {
    let mut buffer = image::DynamicImage::from(image::ImageBuffer::from_pixel(
        tile.size,
        tile.size,
        image::Rgba([0 as u8, 0 as u8, 0 as u8, 0 as u8]),
    ));

    let cropbuffer = source.crop_imm(
        tile.chunk_bounds.0,
        tile.chunk_bounds.1,
        tile.size,
        tile.size,
    );
    cropbuffer
        .pixels()
        .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));

    if tile.size != tile_size {
        buffer = buffer.resize(tile_size, tile_size, FilterType::Nearest);
    }

    return ImageTile {
        image: buffer,
        tile_id: tile.tile_id,
    };
}

/**
 * Generate a compiled tile from a set of source tiles.
 */
pub fn generate_compiled_tile(tile: Tile, tile_size: u32, path: &String, format: &String) -> ImageTile {
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
        image::Rgba([0 as u8, 0 as u8, 0 as u8, 0 as u8]),
    ));

    for source_tile in source_tiles {
        let source_file = format!("{}/{}/{}/{}.{}", path, source_zoom, source_tile.0, source_tile.1, format);
        if let Ok(tile_img) = image::open(source_file) {
            buffer.copy_from(&tile_img, source_tile.2 * tile_size, source_tile.3 * tile_size).unwrap();
        }
    }
    buffer = buffer.resize(tile_size, tile_size, FilterType::Nearest);

    return ImageTile {
        image: buffer,
        tile_id: tile.tile_id,
    };
}