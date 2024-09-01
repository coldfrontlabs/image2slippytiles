use image::{GenericImage, GenericImageView, ImageReader};
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
        let zoom_image = if zoom == max_zoom {
            &source
        } else {
            println!(
                "Resizing image to {} for zoom {} ({})",
                ratio_size, zoom, offset_ratio
            );
            &source.resize(ratio_size, ratio_size, image::imageops::FilterType::Nearest)
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
                let tile = if x * 256 > zoom_image.width() || y * 256 > zoom_image.height() {
                    println!("{} - black", name);
                    black_tile.clone()
                } else if (x * 256) + 256 < zoom_image.width()
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

    memory_check();

    let boundsx = -180.0 + (360.0 * (source.width() as f32 / fullsize as f32));
    let boundsy = 90.0 - (180.0 * (source.height() as f32 / fullsize as f32));

    println!("Bounds: [[90,-180],[{}, {}]]", boundsy, boundsx);

    let boundsx = 1000.0 * (source.width() as f32 / fullsize as f32);
    let boundsy = 1000.0 * (source.height() as f32 / fullsize as f32);

    println!("Bounds: [[0, 0],[{}, {}]]", boundsy, boundsx);
}
