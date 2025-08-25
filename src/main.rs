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

    if let Some(img) = source {
        let json = args.json;
        let tiles = tilechunker::tilechunker(args, 256, 4, img, start_time);
        if json {
            println!("{}", serde_json::to_string_pretty(&tiles).unwrap());
        } else {
            println!("{:#?}", tiles);
        }
    } else {
        println!("Error loading image");
        exit(1);
    }
}
