use clap::Parser;
use std::process::exit;
use serde_json;

mod image2slippytiles;

fn main() {
    let args = image2slippytiles::Cli::parse();
    let source = image2slippytiles::load_image(&args);
    if let Some(img) = source {
        let tiles_res = image2slippytiles::image2slippytiles(args, img);
        if let Ok(tiles) = tiles_res {
            println!("{}", serde_json::to_string_pretty(&tiles).unwrap());
        }
        else {
            println!("Error converting image to tiles");
            exit(1);
        }
    }
    else {
        println!("Error loading image");
        exit(1);
    }
}