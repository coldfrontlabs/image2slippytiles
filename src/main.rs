use clap::Parser;
use image2slippytiles::{chunkable, cli, globals::PEAK_ALLOC, thumbnail, tilechunker};
use std::process::exit;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let mut args = crate::cli::Cli::parse();

    if args.thumbnailfromtiles || args.thumbnailfromzoomifytiles {
        let res = thumbnail::thumbnailfromtiles(&args, None);
        match res {
            Ok(()) => exit(0),
            Err(error) => {
                eprintln!("{}", error);
                exit(1);
            }
        }
    }

    if args.debug {
        args.verbose = true;
    }

    let source = chunkable::load_image(&args);

    match source {
        Ok(img) => {
            if args.test_only {
                println!("Image \"{}\" can be loaded.", args.filename);
                exit(0)
            }

            let json = args.json;

            let tiles_res = tilechunker::tilechunker(args, 256, 4, img, start_time).await;
            match tiles_res {
                Ok(tiles) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tiles).unwrap());
                    } else {
                        println!("{:#?}", tiles);
                    }
                }
                Err(error) => {
                    eprintln!("{}", error);
                    exit(1);
                }
            }
        }
        Err(message) => {
            eprintln!("Error loading image: {}", message);
            eprintln!("Peak memory usage was: {}", PEAK_ALLOC.peak_usage_as_mb());
            exit(1);
        }
    }
}
