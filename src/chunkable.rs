use crate::cli::Cli;
use crate::dynamicimage::load_dynamic_image;
use crate::metadata::*;
use crate::openslide::load_openslide_image;
use image::DynamicImage;
use openslide_rs::OpenSlide;
use std::path::Path;
pub enum ChunkableImageSource {
    DynamicImage(DynamicImage),
    Slide(OpenSlide),
}

impl ChunkSource for ChunkableImageSource {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32, colour: &[u8; 4]) -> DynamicImage {
        match self {
            ChunkableImageSource::DynamicImage(img) => img.get_chunk(chunk_id, chunk_width, chunk_height, colour),
            ChunkableImageSource::Slide(slide) => slide.get_chunk(chunk_id, chunk_width, chunk_height, colour),
        }
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        match self {
            ChunkableImageSource::DynamicImage(img) => img.get_image_metadata(),
            ChunkableImageSource::Slide(slide) => slide.get_image_metadata(),
        }
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        match self {
            ChunkableImageSource::DynamicImage(img) => img.get_slide_metadata(),
            ChunkableImageSource::Slide(slide) => slide.get_slide_metadata(),
        }
    }
}

pub trait ChunkSource {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32, colour: &[u8; 4]) -> DynamicImage;
    fn get_image_metadata(&self) -> ImageMetadata;
    fn get_slide_metadata(&self) -> Option<SlideMetadata>;

    fn get_full_image(&self, colour: &[u8; 4]) -> DynamicImage {
        return self.get_chunk((0, 0), self.get_image_metadata().width, self.get_image_metadata().height, colour);
    }
}

pub fn load_image(args: &Cli) -> Option<ChunkableImageSource> {
    let path = Path::new(&args.filename);

    if path.extension().unwrap() == "svs" || path.extension().unwrap().to_str().unwrap() == "dcm" {
        return load_openslide_image(args);
    } else {
        return load_dynamic_image(args);
    }
}