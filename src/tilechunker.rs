use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use std::fs;
use crate::metadata::*;
use crate::globals::PEAK_ALLOC;
use crate::cli::Cli;
use crate::chunkable::*;

#[derive(Debug)]

pub struct Tile {
    chunk_id: (u32, u32),
    tile_id: (u32, u32, u32),
    size: u32,
    chunk_bounds: (u32, u32, u32, u32),
    image_bounds: (u32, u32, u32, u32),
}

pub struct ImageTile {
    pub tile_id: (u32, u32, u32),
    pub image: DynamicImage,
}

impl ImageTile {
    pub fn save(&self, path: &str) {
        fs::create_dir(format!("{}/{}", path, self.tile_id.0)).unwrap_or_default();
        fs::create_dir(format!("{}/{}/{}", path, self.tile_id.0, self.tile_id.1)).unwrap_or_default();
        let name: String = format!("{}/{}/{}/{}.png", path, self.tile_id.0, self.tile_id.1, self.tile_id.2);
        self.image.save_with_format(name, image::ImageFormat::Png)
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
                            chunk_id: chunk_id,
                            tile_id: tile_id,
                            size: tile_size,
                            chunk_bounds: tile_bounds_in_chunk,
                            image_bounds: tile_bounds_in_image,
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
                            generate_partial_tile(&chunk, tile, final_tile_size).save(&path);
                        }
                        else {
                            if args.debug {
                                println!("Full tile");
                            }
                            generate_full_tile(&chunk, tile, final_tile_size).save(&path);
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

    TileMetadata {
        min_zoom: max_chunk_zoom,
        max_zoom: max_zoom,
        bounds: [
            0.0,
            0.0,
            -1.0 * image_metadata.height as f32 / u32::pow( 2, max_zoom) as f32,
            image_metadata.width as f32 /u32::pow( 2, max_zoom) as f32,
        ],
        peak_memory: PEAK_ALLOC.peak_usage_as_mb(),
        image_metadata: source.get_image_metadata(),
        slide_metadata: source.get_slide_metadata(),
    }

}

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
    println!("Selected from {}x{} at size {}x{}, got size {}x{}", tile.chunk_bounds.0, tile.chunk_bounds.1, tile.size, tile.size, cropbuffer.width(), cropbuffer.height());

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