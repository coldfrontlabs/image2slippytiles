use crate::chunkable::*;
use crate::cli::Cli;
use crate::metadata::*;
use image::{DynamicImage, GenericImage, GenericImageView};
use openslide_rs::*;
use std::path::Path;

pub fn load_openslide_image(args: &Cli) -> Option<ChunkableImageSource> {
    let path = Path::new(&args.filename);
    let slide = OpenSlide::new(path).unwrap();

    if args.verbose {
        println!("Properties: {:#?}", slide.properties().openslide_properties);
    }

    return Some(ChunkableImageSource::Slide(slide));
}

impl ChunkSource for OpenSlide {
    fn get_chunk(
        &self,
        chunk_id: (u32, u32),
        chunk_width: u32,
        chunk_height: u32,
        default_colour: &[u8; 4],
    ) -> DynamicImage {
        let dimensions = self.get_level_dimensions(0).unwrap();

        let mut size_w = dimensions.w - (chunk_id.0 * chunk_width);
        let mut size_h = dimensions.h - (chunk_id.1 * chunk_height);

        if size_w >= chunk_width {
            size_w = chunk_width;
        }

        if size_h >= chunk_height {
            size_h = chunk_height;
        }

        let img = self
            .read_image_rgba(&Region {
                address: Address {
                    x: chunk_id.0 * chunk_width,
                    y: chunk_id.1 * chunk_height,
                },
                level: 0,
                size: Size {
                    w: size_w,
                    h: size_h,
                },
            })
            .unwrap();

        if (size_w < chunk_width) || (size_h < chunk_height) {
            // println!("Chunk {}x{} is a partial chunk: {}x{}", chunk_id.0, chunk_id.1, size_w, size_h);
            let mut buffer = image::DynamicImage::from(image::ImageBuffer::from_pixel(
                chunk_width,
                chunk_height,
                image::Rgba(default_colour.clone()),
            ));

            let source = DynamicImage::ImageRgba8(img);
            source
                .pixels()
                .for_each(|(x, y, pixel)| buffer.put_pixel(x, y, pixel));
            return buffer;
        }
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
            mpp_x: *self
                .properties()
                .openslide_properties
                .mpp_x
                .clone()
                .get_or_insert(0.0),
            mpp_y: *self
                .properties()
                .openslide_properties
                .mpp_y
                .clone()
                .get_or_insert(0.0),
        });
    }
}
