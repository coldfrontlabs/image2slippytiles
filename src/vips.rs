use crate::chunkable::*;
use crate::cli::Cli;
use crate::metadata::*;
use libvips::{ops, VipsImage, VipsApp};
use image::{ImageBuffer, DynamicImage};

pub fn load_vips_image(args: &Cli) -> Option<ChunkableImageSource> {
    let app = VipsApp::new("Test Libvips", false).expect("Cannot initialize libvips");
    //set number of threads in libvips's threadpool
    app.concurrency_set(2);
    // loads an image from file
    let img_res = VipsImage::new_from_file(&args.filename);

    if let Ok( img) = img_res {
        if args.verbose {
            println!("format: {:?}", img.get_format());
        }
        return Some(ChunkableImageSource::Vips(img));
    }

    if let Err(e) = img_res {
        println!("Error: {}", e);
    }

    return None;
}

impl ChunkSource for VipsImage {

    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {
        let chunk = ops::extract_area(
            self,
            (chunk_id.0 * chunk_width) as i32,
            (chunk_id.1 * chunk_height) as i32,
            chunk_width as i32,
            chunk_height as i32,
        ).unwrap();
    
        return DynamicImage::ImageRgba8(ImageBuffer::from_vec(
            chunk.get_width() as u32,
            chunk.get_height() as u32,
            chunk.image_write_to_memory(),
        ).unwrap());
    }

    fn get_image_metadata(&self) -> ImageMetadata {
        return ImageMetadata {
            width: self.get_width() as u32,
            height: self.get_height() as u32,
        };
    }

    fn get_slide_metadata(&self) -> Option<SlideMetadata> {
        return None;
    }
}
