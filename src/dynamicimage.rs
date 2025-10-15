use crate::chunkable::*;
use crate::cli::Cli;
use crate::metadata::*;
use image::{DynamicImage, ImageReader};
use std::path::Path;

pub fn load_dynamic_image(args: &Cli) -> Result<ChunkableImageSource, String> {
    let path = Path::new(&args.filename);

    let mut img = ImageReader::open(path).unwrap();

    if args.verbose {
        println!("format: {:?}", img.format());
    }

    img.no_limits();

    let source = img.decode();

    match source {
        Ok(source) => Ok(ChunkableImageSource::DynamicImage(source)),
        Err(e) => {
            Err(format!("Error: {}", e))
        }
    }
}

impl ChunkSource for DynamicImage {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {
        self.crop_imm(
            chunk_id.0 * chunk_width,
            chunk_id.1 * chunk_height,
            chunk_width,
            chunk_height,
        )
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        ImageMetadata {
            width: self.width(),
            height: self.height(),
        }
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        None
    }
}
