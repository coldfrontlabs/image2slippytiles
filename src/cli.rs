use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The input image
    pub filename: String,

    /// Verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Debug output
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Output directory ('tiles' in the current directory if no option provided).
    #[arg(short = 'o', long, default_value = "./tiles")]
    pub output: String,

    /// Output memory usage
    #[arg(short = 'm', long)]
    pub memory: bool,

    /// Starting zoom level
    #[arg(short = 'z', long, default_value = "0")]
    pub zoom: u32,

    /// Output JSON metadata
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Output type (png, jpg, webp)
    #[arg(short = 't', long, default_value = "png")]
    pub format: String,

    /// Background color hex code: formatted as #RRGGBBAA
    #[arg(short = 'c', long, default_value = "#00000000")]
    pub colour: String,

    /// Generate thumbnail.
    #[arg(short = 'T', long)]
    pub thumbnail: bool,

    /// Thumbnail size.
    #[arg(short = 's', long, default_value = "512")]
    pub thumbnailsize: u32,

    /// Generate a thumbnail from an existing tileset.
    #[arg(long)]
    pub thumbnailfromtiles: bool,

    /// Generate a thumbnail from an existing zoomify tileset.
    #[arg(long)]
    pub thumbnailfromzoomifytiles: bool,
}
