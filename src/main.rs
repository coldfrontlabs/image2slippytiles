use clap::Parser;
use image2slippytiles::{chunkable, cli, tilechunker};
use serde_json;
use std::process::exit;
use std::time::Instant;

fn main() {
    let start_time = Instant::now();
    let args = crate::cli::Cli::parse();
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
