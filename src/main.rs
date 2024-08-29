use image::ImageReader;
use std::fs;

fn main() {
    //let mut img = ImageReader::open("../image2tiles.php/PIA25162.png").unwrap();
    let mut img = ImageReader::open("../image2tiles.php/PIA23623_hires.png").unwrap();
    img.no_limits();
    println!("format: {:?}", img.format());
    let mut source = img.decode().unwrap();

    let max = std::cmp::max(source.width(), source.height());
    let scale = (max as f32 / 256.0).ceil() as u32;
    let max_zoom = (scale as f32).log2().ceil() as u32;
    println!("scale: {}, max_zoom: {}", scale, max_zoom);

    let dir = "tiles";
    fs::create_dir(dir).unwrap_or_default(); 

    for zoom in 0..max_zoom {
        if zoom > 2 {
            panic!("zoom too high");
        }
        fs::create_dir(format!("{}/{}", dir, zoom)).unwrap_or_default();
        let mzoom = max_zoom - zoom;

        let zoom_image = source.resize(256 * u32::pow(2, zoom), 256 * u32::pow(2, zoom), image::imageops::FilterType::Nearest);

        println!("zoom: {}, size: {}x{}", zoom, zoom_image.width(), zoom_image.height());
        for x in 0..u32::pow(2, zoom) {
            fs::create_dir(format!("{}/{}/{}", dir, zoom, x)).unwrap_or_default();
            for y in 0..u32::pow(2, zoom) {
                let mut tile = image::DynamicImage::from(image::ImageBuffer::from_pixel(256, 256, image::Rgb([0 as u16, 0 as u16, 0 as u16])));
                let name = format!("{}/{}/{}/{}.webp", dir, zoom, x, y);
                let tile = zoom_image.crop_imm(x * 256 * u32::pow(2, mzoom), y * 256 * u32::pow(2, mzoom), 256, 256);
                println!("{}", name);
                tile.save_with_format(name, image::ImageFormat::WebP).unwrap();
            }
        }

    }


}
