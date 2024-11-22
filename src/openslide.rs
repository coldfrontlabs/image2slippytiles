use std::path::Path;
use openslide_rs::*;
use image::DynamicImage;
use crate::metadata::*;
use crate::cli::Cli;
use crate::chunkable::*;

pub fn load_openslide_image(args: &Cli) -> Option<ChunkableImageSource> {
    let path = Path::new(&args.filename);
    let slide = OpenSlide::new(path).unwrap();

    if args.verbose {
        println!("Properties: {:#?}", slide.properties().openslide_properties);
    }

    return Some(ChunkableImageSource::Slide(slide));
}


impl ChunkSource for OpenSlide {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {
        let img = self.read_image_rgba(&Region {
            address: Address {
                x: chunk_id.0 * chunk_width,
                y: chunk_id.1 * chunk_height,
            },
            level: 0,
            size: Size { w: chunk_width, h: chunk_height },
        }).unwrap();
        return DynamicImage::ImageRgba8(img);
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        let dimensions = self.get_level_dimensions(0).unwrap();
        return ImageMetadata {
            width: dimensions.w,
            height: dimensions.h,
        };
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        return Some(SlideMetadata {
            mpp_x: *self.properties().openslide_properties.mpp_x.clone().get_or_insert(0.0),
            mpp_y: *self.properties().openslide_properties.mpp_y.clone().get_or_insert(0.0),
        });
    }

}