use std::path::Path;
use image::{DynamicImage, ImageReader};
use crate::metadata::*;
use crate::cli::Cli;
use crate::chunkable::*;

pub fn load_dynamic_image(args: &Cli) -> Option<ChunkableImageSource> {
    let path = Path::new(&args.filename);

    let mut img = ImageReader::open(&path).unwrap();

    if args.verbose {
        println!("format: {:?}", img.format());
    }

    img.no_limits();

    let source = img.decode();
    
    match source {
        Ok(source) => Some(ChunkableImageSource::DynamicImage(source)),
        Err(e) => {
            println!("Error: {}", e);
            None
        }
    }
}

impl ChunkSource for DynamicImage {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32, _: &[u8; 4]) -> DynamicImage {
        return self.crop_imm(chunk_id.0 * chunk_width, chunk_id.1 * chunk_height, chunk_width, chunk_height);
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        return ImageMetadata {
            width: self.width(),
            height: self.height(),
        };
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        return None;
    }
}