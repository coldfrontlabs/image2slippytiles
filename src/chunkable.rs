use crate::cli::Cli;
use crate::dynamicimage::load_dynamic_image;
use crate::metadata::*;
use crate::openslide::load_openslide_image;
use image::DynamicImage;
use openslide_rs::OpenSlide;
use std::path::Path;
use libvips::VipsImage;
use image::ImageFormat;

const OPENSLIDE_FORMATS: [&str; 12] = [
    "svs", "tif", "dcm", "vms", "vmu", "ndpi", "scn", "mrxs", "tiff", "svslide", "bif", "czi",
];

pub enum ChunkableImageSource {
    DynamicImage(DynamicImage),
    Slide(Box<OpenSlide>),
    //Vips(VipsImage),
}

impl ChunkSource for ChunkableImageSource {
    fn get_threads(&self) -> usize {
        match self {
            ChunkableImageSource::DynamicImage(_) => return 1,
            ChunkableImageSource::Slide(_) => return 4,
        }
    }

    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {
        match self {
            ChunkableImageSource::DynamicImage(img) => {
                img.get_chunk(chunk_id, chunk_width, chunk_height)
            }
            ChunkableImageSource::Slide(slide) => {
                slide.get_chunk(chunk_id, chunk_width, chunk_height)
            }
            ChunkableImageSource::Vips(vips) => {
                vips.get_chunk(chunk_id, chunk_width, chunk_height)
            }
        }
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        match self {
            ChunkableImageSource::DynamicImage(img) => img.get_image_metadata(),
            ChunkableImageSource::Slide(slide) => slide.get_image_metadata(),
            ChunkableImageSource::Vips(vips) => vips.get_image_metadata(),
        }
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        match self {
            ChunkableImageSource::DynamicImage(img) => img.get_slide_metadata(),
            ChunkableImageSource::Slide(slide) => slide.get_slide_metadata(),
            ChunkableImageSource::Vips(vips) => vips.get_slide_metadata(),
        }
    }
}

pub trait ChunkSource {
    fn get_threads(&self) -> usize;
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
            if OPENSLIDE_FORMATS.contains(&extension.to_str().unwrap()) {
                let res = load_openslide_image(args);
                if res.is_err() {
                    load_dynamic_image(args, Some(ImageFormat::Tiff))
                } else {
                    res
                }
            }
            
            else {
                load_dynamic_image(args, None)
            }
        }
        None => Err(format!("Unknown file type in file: {}", args.filename)),
    }
}
