use crate::chunkable::*;
use crate::cli::Cli;
use crate::globals::PEAK_ALLOC;
use crate::metadata::*;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use openslide_rs::*;
use std::fs;

pub fn image2slippytiles(args: Cli, image_process: ImageProcess) -> Result<TileMetadata, String> {
    let source = image_process.image.get_full_image();
    if args.verbose {
        println!("Image: {}x{}", source.width(), source.height());
    }

    let max = std::cmp::max(source.width(), source.height());
    let scale = (max as f32 / 256.0).ceil() as u32;
    let max_zoom = (scale as f32).log2().ceil() as u32;

    if max_zoom < args.zoom {
        return Err(format!(
            "The max zoom on the source image can only be zoomed to {}. {} is too high.",
            max_zoom, args.zoom,
        ));
    }

    let mut end_zoom = max_zoom;
    if end_zoom < args.zoom {
        return Err(format!(
            "The end zoom level must be greater than the start zoom level. {} is less than {}.",
            end_zoom, args.zoom
        ));
    }
    if end_zoom > max_zoom {
        println!(
            "Warning: The max zoom level is {} (less the the requested end zoom level {}).  Using {} as the end zoom level.",
            max_zoom, end_zoom, max_zoom
        );
        end_zoom = max_zoom;
    }

    let fullsize = u32::pow(2, max_zoom) * 256;
    if args.verbose {
        println!(
            "Max image resolution: {}, Full zoom level resolution: {}",
            max, fullsize
        );
    }
    let offset_ratio = max as f32 / fullsize as f32;
    if args.verbose {
        println!(
            "scale: {}, max_zoom: {}, offset_ratio: {}",
            scale, max_zoom, offset_ratio
        );
    }

    let dir = args.output.unwrap_or("tiles".to_string());
    fs::create_dir(&dir).unwrap_or_default();

    let black_tile = image::DynamicImage::from(image::ImageBuffer::from_pixel(
        256,
        256,
        image::Rgba([0 as u8, 0 as u8, 0 as u8, 0 as u8]),
    ));

    for zoom in args.zoom..end_zoom + 1 {
        fs::create_dir(format!("{}/{}", dir, zoom)).unwrap_or_default();

        let ratio_size = if zoom == max_zoom {
            max
        } else {
            (256.0 * u32::pow(2, zoom) as f32 * offset_ratio) as u32
        };
        let tile_size_at_zoom = 256 * u32::pow(2, max_zoom - zoom);
        if args.verbose {
            println!(
                "Processing zoom: {}, ratio_size: {}, tile_size_at_zoom: {}",
                zoom, ratio_size, tile_size_at_zoom
            );
        }
        if max_zoom / 2 > zoom {
            // When zoomed out, it's faster to resize the entire image and chunk it out.
            zoom_then_crop(
                &source,
                &black_tile,
                &dir,
                zoom,
                max_zoom,
                ratio_size,
                args.verbose,
                args.debug,
            );
        } else {
            // At closer to the native image resolution, it requires less memory to crop first, but more CPU.
            crop_then_zoom(
                &source,
                &dir,
                zoom,
                tile_size_at_zoom,
                args.verbose,
                args.debug,
            );
        }
    }
    return Ok(TileMetadata {
        min_zoom: args.zoom,
        max_zoom: end_zoom,
        bounds: [
            0.0,
            0.0,
            -1.0 * source.height() as f32 / u32::pow(2, max_zoom) as f32,
            source.width() as f32 / u32::pow(2, max_zoom) as f32,
        ],
        image_type: args.format,
        image_metadata: image_process.image_metadata,
        slide_metadata: image_process.slide_metadata,
        peak_memory: PEAK_ALLOC.peak_usage_as_mb(),
    });

    /*
    // Determine bounds of the slippy map.
    let boundsx = -180.0 + (360.0 * (source.width() as f32 / fullsize as f32));
    let boundsy = 90.0 - (180.0 * (source.height() as f32 / fullsize as f32));

    println!("Bounds: [[90,-180],[{}, {}]]", boundsy, boundsx);

    let boundsx = 1000.0 * (source.width() as f32 / fullsize as f32);
    let boundsy = 1000.0 * (source.height() as f32 / fullsize as f32);

    println!("Bounds: [[0, 0],[{}, {}]]", boundsy, boundsx);
    */
}

fn zoom_then_crop(
    source: &DynamicImage,
    black_tile: &DynamicImage,
    dir: &String,
    zoom: u32,
    max_zoom: u32,
    ratio_size: u32,
    verbose: bool,
    debug: bool,
) {
    let zoom_image = if zoom == max_zoom {
        &source
    } else {
        &source.resize(ratio_size, ratio_size, FilterType::Nearest)
    };
    if verbose {
        println!(
            "Zoom Image at {}: {}x{}",
            zoom,
            zoom_image.width(),
            zoom_image.height()
        );
    }

    for x in 0..u32::pow(2, zoom) {
        for y in 0..u32::pow(2, zoom) {
            let name = format!("{}/{}/{}/{}.png", dir, zoom, x, y);
            if x * 256 > zoom_image.width() || y * 256 > zoom_image.height() {
                continue;
            }
            let tile =
                if (x * 256) + 256 < zoom_image.width() && (y * 256) + 256 < zoom_image.height() {
                    if debug {
                        println!("Zoom-then-crop: Genered tile {} by cropping", name);
                    }
                    zoom_image.crop_imm(x * 256, y * 256, 256, 256)
                } else {
                    let mut buffer = black_tile.clone();
                    let cropbuffer = zoom_image.crop_imm(x * 256, y * 256, 256, 256);
                    if debug {
                        println!(
                            "Zoom-then-crop: Genered tile {} by partial cropping {}x{}",
                            name,
                            cropbuffer.width(),
                            cropbuffer.height()
                        );
                    }
                    cropbuffer
                        .pixels()
                        .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));
                    buffer
                };
            fs::create_dir(format!("{}/{}/{}", dir, zoom, x)).unwrap_or_default();
            tile.save_with_format(name, image::ImageFormat::Png)
                .unwrap();
        }
    }
}

fn crop_then_zoom(
    source: &DynamicImage,
    dir: &String,
    zoom: u32,
    tile_size_at_zoom: u32,
    verbose: bool,
    debug: bool,
) {
    if verbose {
        println!("Tile size at zoom {}: {}", zoom, tile_size_at_zoom);
    }
    for x in 0..u32::pow(2, zoom) {
        for y in 0..u32::pow(2, zoom) {
            let name = format!("{}/{}/{}/{}.png", dir, zoom, x, y);

            // Skip tiles outside image.
            if x * tile_size_at_zoom > source.width() || y * tile_size_at_zoom > source.height() {
                continue;
            }

            let tile = if (x * tile_size_at_zoom) + tile_size_at_zoom < source.width()
                && (y * tile_size_at_zoom) + tile_size_at_zoom < source.height()
            {
                if debug {
                    println!(
                        "Crop-then-zoom: Generated tile {} ({}x{}) by cropping",
                        name, tile_size_at_zoom, tile_size_at_zoom
                    );
                }
                source
                    .crop_imm(
                        x * tile_size_at_zoom,
                        y * tile_size_at_zoom,
                        tile_size_at_zoom,
                        tile_size_at_zoom,
                    )
                    .resize(256, 256, FilterType::Nearest)
            } else {
                let mut buffer = image::DynamicImage::from(image::ImageBuffer::from_pixel(
                    tile_size_at_zoom,
                    tile_size_at_zoom,
                    image::Rgba([0 as u8, 0 as u8, 0 as u8, 0 as u8]),
                ));
                let cropbuffer = source.crop_imm(
                    x * tile_size_at_zoom,
                    y * tile_size_at_zoom,
                    tile_size_at_zoom,
                    tile_size_at_zoom,
                );
                if debug {
                    println!(
                        "Crop-then-zoom: Generated tile {} ({}x{}) by partial cropping",
                        name,
                        cropbuffer.width(),
                        cropbuffer.height()
                    );
                }
                cropbuffer
                    .pixels()
                    .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));
                buffer.resize(256, 256, FilterType::Nearest)
            };
            fs::create_dir(format!("{}/{}/{}", dir, zoom, x)).unwrap_or_default();
            tile.save_with_format(name, image::ImageFormat::Png)
                .unwrap();
        }
    }
}
