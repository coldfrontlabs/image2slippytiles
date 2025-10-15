use clap::Parser;
use image2slippytiles::{chunkable, cli, thumbnail, tilechunker};
use std::process::exit;
use std::time::Instant;

fn main() {
    let start_time = Instant::now();
    let mut args = crate::cli::Cli::parse();

    if args.thumbnailfromtiles || args.thumbnailfromzoomifytiles {
        thumbnail::thumbnailfromtiles(args);
        exit(0);
    }

    if args.debug {
        args.verbose = true;
    }

    let source = chunkable::load_image(&args);

    //if let Ok(img) = source {
    match source {
        Ok(img) => {
            let json = args.json;
            let tiles_res = tilechunker::tilechunker(args, 256, 4, img, start_time);
            match tiles_res {
                Ok(tiles) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tiles).unwrap());
                    } else {
                        println!("{:#?}", tiles);
                    }
                }
                Err(message) => {
                    println!("{}", message);
                    exit(1);
                }
            }
        },
        Err(message) => {
            println!("Error loading image: {}", message);
            exit(1);
        }
    }
}
