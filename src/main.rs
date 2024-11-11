use clap::Parser;
mod image2slippytiles;

fn main() {
    let args = image2slippytiles::Cli::parse();
    let source = image2slippytiles::load_image(&args);
    match source {
        Some(img) => image2slippytiles::image2slippytiles(args, img),
        None => println!("Error loading image"),
    };
}