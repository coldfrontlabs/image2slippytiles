use serde_json;
use std::process::exit;

mod image2slippytiles;

fn main() {
    let args = crate::cli::Cli::parse();
    let source = image2slippytiles::load_image(&args);
    if let Some(img) = source {
        let json = args.json;
        let tiles_res = image2slippytiles::image2slippytiles(args, img);
        if let Ok(tiles) = tiles_res {
            if json {
                println!("{}", serde_json::to_string_pretty(&tiles).unwrap());
            } else {
                println!("{:#?}", tiles);
            }
        } else {
            println!("Error converting image to tiles");
            exit(1);
        }
    } else {
        println!("Error loading image");
        exit(1);
    }
}
