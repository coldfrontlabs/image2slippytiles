use crate::cli::Cli;
use crate::dynamicimage::load_dynamic_image;
use crate::metadata::*;
use crate::openslide::load_openslide_image;
use image::DynamicImage;
use openslide_rs::OpenSlide;
use std::path::Path;
pub enum ChunkableImageSource {
    DynamicImage(DynamicImage),
    Slide(Box<OpenSlide>),
}

impl ChunkSource for ChunkableImageSource {
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {
        match self {
            ChunkableImageSource::DynamicImage(img) => {
                img.get_chunk(chunk_id, chunk_width, chunk_height)
            }
            ChunkableImageSource::Slide(slide) => {
                slide.get_chunk(chunk_id, chunk_width, chunk_height)
            }
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
    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage;
    fn get_image_metadata(&self) -> ImageMetadata;
    fn get_slide_metadata(&self) -> Option<SlideMetadata>;

    fn get_full_image(&self) -> DynamicImage {
        self.get_chunk(
            (0, 0),
            self.get_image_metadata().width,
            self.get_image_metadata().height,
        )
    }
}

pub fn load_image(args: &Cli) -> Result<ChunkableImageSource, String> {
    let path = Path::new(&args.filename);
    let extension_res = path.extension();

    match extension_res {
        Some(extension) => {
            if extension == "svs" || extension == "dcm" {
                load_openslide_image(args)
            } else {
                load_dynamic_image(args)
            }
        }
        None => Err(format!("Unknown file type in file: {}", args.filename)),
    }
}
