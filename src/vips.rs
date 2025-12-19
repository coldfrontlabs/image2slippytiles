use crate::chunkable::*;
use crate::cli::Cli;
use crate::metadata::*;
use rs_vips::{ops, Vips, VipsImage};
use std::io::Cursor;
use image::{ImageReader, ImageFormat, DynamicImage};

pub fn load_vips_image(args: &Cli, format: Option<ImageFormat>) -> Result<ChunkableImageSource, String> {
    Vips::init("Test Libvips").expect("Cannot initialize libvips");
    Vips::leak_set(false);
    Vips::concurrency_set(2);

    match VipsImage::new_from_file(&args.filename) {
        Ok(img) =>  Ok(ChunkableImageSource::Vips(img)),
        Err(e) => Err(e.to_string())
    }
}

impl ChunkSource for VipsImage {
    fn get_threads(&self) -> usize {
        1
    }

    fn get_chunk(&self, chunk_id: (u32, u32), chunk_width: u32, chunk_height: u32) -> DynamicImage {


        let mut new_width = chunk_width;
        let width = self.get_width() as u32;
        let height = self.get_height() as u32;



        if width < (chunk_id.0 * chunk_width) + chunk_width {
            new_width = width - (chunk_id.0 * chunk_width);
        }

        let mut new_height = chunk_height;
        if height < (chunk_id.1 * chunk_height) + chunk_height {
            new_height = height - (chunk_id.1 * chunk_height);
        }
/*
        eprintln!("Extract:\nLeft: {}\nTop: {}\nHeight: {}\nWidth: {}\nImage height: {}\nImage width: {}\nTile width: {}\nTile height: {}",
            (chunk_id.0 * chunk_width) as i32,
            (chunk_id.1 * chunk_height) as i32,
            chunk_height as i32,
            chunk_width as i32,
            height,
            width,
            new_width,
            new_height,
        );*/
        eprintln!("Chunk fragment:\nX: {} - {} (of {})\nY: {} - {} (of {})\n",
         (chunk_id.0 * chunk_width),
         (chunk_id.0 * chunk_width) + new_width,
         width,
        (chunk_id.1 * chunk_height),
         (chunk_id.1 * chunk_height) + new_height,
         height,
        );

        let chunk_res = self.extract_area(
            (chunk_id.0 * chunk_width) as i32,
            (chunk_id.1 * chunk_height) as i32,
            new_width as i32,
            new_height as i32,
        );


        if let Err(err) = chunk_res {
            eprintln!("Got VIPS Error: {}", err.to_string());
            panic!();
        }
        else {
            let chunk = chunk_res.unwrap();
        eprintln!("Write image to memory");
        let data = chunk.pngsave_buffer();
        eprintln!("Building buffer");
        return ImageReader::new(Cursor::new(data.unwrap())).with_guessed_format().unwrap().decode().unwrap()

        }
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
