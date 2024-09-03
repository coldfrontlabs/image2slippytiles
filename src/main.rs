use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};
use image::imageops::FilterType;
use peak_alloc::PeakAlloc;
use std::fs;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn memory_check() {
    let current_mem = PEAK_ALLOC.current_usage_as_mb();
    println!("This program currently uses {} MB of RAM.", current_mem);
    let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
    println!("The max amount that was used {} MB", peak_mem);
}

fn main() {
    let mut img = ImageReader::open("../image2tiles.php/PIA23623_hires.png").unwrap();
    img.no_limits();
    println!("format: {:?}", img.format());
    let source = img.decode().unwrap();
    println!("Image: {}x{}", source.width(), source.height());
    let max = std::cmp::max(source.width(), source.height());
    let scale = (max as f32 / 256.0).ceil() as u32;
    let max_zoom = (scale as f32).log2().ceil() as u32;
    let fullsize = u32::pow(2, max_zoom) * 256;
    println!("max: {}, full: {}", max, fullsize);
    let offset_ratio = max as f32 / fullsize as f32;
    println!(
        "scale: {}, max_zoom: {}, offset_ratio: {}",
        scale, max_zoom, offset_ratio
    );

    let dir = "tiles";
    fs::create_dir(dir).unwrap_or_default();

    let black_tile = image::DynamicImage::from(image::ImageBuffer::from_pixel(
        256,
        256,
        image::Rgb([0 as u8, 0 as u8, 0 as u8]),
    ));

    for zoom in 0..max_zoom + 1 {
        fs::create_dir(format!("{}/{}", dir, zoom)).unwrap_or_default();

        let ratio_size = if zoom == max_zoom {
            max
        } else {
            (256.0 * u32::pow(2, zoom) as f32 * offset_ratio) as u32
        };
        let tile_size_at_zoom = 256 * u32::pow(2, max_zoom - zoom);
        println!(
            "Zoom: {}, ratio_size: {}, tile_size_at_zoom: {}",
            zoom, ratio_size, tile_size_at_zoom
        );
        if max_zoom / 2 > zoom {
            // When zoomed out, it's faster to resize the entire image and chunk it out.
            zoom_then_crop(&source, &black_tile, dir, zoom, max_zoom, offset_ratio, ratio_size);
        } else {
            // At closer to the native image resolution, it requires less memory to crop first, but more CPU.
            crop_then_zoom(&source, dir, zoom, tile_size_at_zoom);
        }
        memory_check()
    }

    let boundsx = -180.0 + (360.0 * (source.width() as f32 / fullsize as f32));
    let boundsy = 90.0 - (180.0 * (source.height() as f32 / fullsize as f32));

    println!("Bounds: [[90,-180],[{}, {}]]", boundsy, boundsx);

    let boundsx = 1000.0 * (source.width() as f32 / fullsize as f32);
    let boundsy = 1000.0 * (source.height() as f32 / fullsize as f32);

    println!("Bounds: [[0, 0],[{}, {}]]", boundsy, boundsx);
}

fn zoom_then_crop(
    source: &DynamicImage,
    black_tile: &DynamicImage,
    dir: &str,
    zoom: u32,
    max_zoom: u32,
    offset_ratio: f32,
    ratio_size: u32,
) {
    let zoom_image = if zoom == max_zoom {
        &source
    } else {
        println!(
            "Resizing image to {} for zoom {} ({})",
            ratio_size, zoom, offset_ratio
        );
        &source.resize(ratio_size, ratio_size, FilterType::Nearest)
    };
    println!(
        "Zoom Image at {}: {}x{}",
        zoom,
        zoom_image.width(),
        zoom_image.height()
    );

    for x in 0..u32::pow(2, zoom) {
        fs::create_dir(format!("{}/{}/{}", dir, zoom, x)).unwrap_or_default();
        for y in 0..u32::pow(2, zoom) {
            let name = format!("{}/{}/{}/{}.png", dir, zoom, x, y);
            if x * 256 > zoom_image.width() || y * 256 > zoom_image.height() {
                continue;
            }
            let tile = if (x * 256) + 256 < zoom_image.width()
                && (y * 256) + 256 < zoom_image.height()
            {
                //println!("{} - crop", name);
                zoom_image.crop_imm(x * 256, y * 256, 256, 256)
            } else {
                let mut buffer = black_tile.clone();
                let cropbuffer = zoom_image.crop_imm(x * 256, y * 256, 256, 256);
                //println!("{} - partial crop {}x{}", name, cropbuffer.width(), cropbuffer.height());
                cropbuffer
                    .pixels()
                    .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));
                buffer
            };
            tile.save_with_format(name, image::ImageFormat::Png)
                .unwrap();
        }
    }
}

fn crop_then_zoom(
    source: &DynamicImage,
    dir: &str,
    zoom: u32,
    tile_size_at_zoom: u32,
) {
    println!("Tile size at zoom {}: {}", zoom, tile_size_at_zoom);
    for x in 0..u32::pow(2, zoom) {
        fs::create_dir(format!("{}/{}/{}", dir, zoom, x)).unwrap_or_default();
        for y in 0..u32::pow(2, zoom) {
            let name = format!("{}/{}/{}/{}.png", dir, zoom, x, y);

            // Skip tiles outside image.
            if x * tile_size_at_zoom > source.width() || y * tile_size_at_zoom > source.height() {
                continue;
            }

            let tile = if (x * tile_size_at_zoom) + tile_size_at_zoom < source.width()
            && (y * tile_size_at_zoom) + tile_size_at_zoom < source.height()
            {
                //println!("{} ({}x{}) - crop", name, tile_size_at_zoom, tile_size_at_zoom);
                source.crop_imm(x * tile_size_at_zoom, y * tile_size_at_zoom, tile_size_at_zoom, tile_size_at_zoom).resize(256, 256, FilterType::Nearest)
            } else {
                let mut buffer =  image::DynamicImage::from(image::ImageBuffer::from_pixel(
                    tile_size_at_zoom,
                    tile_size_at_zoom,
                    image::Rgb([0 as u8, 0 as u8, 0 as u8]),
                ));
                let cropbuffer = source.crop_imm(x * tile_size_at_zoom, y * tile_size_at_zoom, tile_size_at_zoom, tile_size_at_zoom);
                //println!("{} - partial crop {}x{}", name, cropbuffer.width(), cropbuffer.height());
                cropbuffer
                    .pixels()
                    .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));
                buffer.resize(256, 256, FilterType::Nearest)
            };
            tile.save_with_format(name, image::ImageFormat::Png)
                .unwrap();
        }
    }
}