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
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Output memory usage
    #[arg(short = 'm', long)]
    pub memory: bool,

    /// Starting zoom level
    #[arg(short = 'z', long, default_value = "0")]
    pub zoom: u32,

    /// End zoom level
    #[arg(short = 'x', long)]
    pub end_zoom: Option<u32>,

    /// Output JSON metadata
    #[arg(short = 'j', long)]
    pub json: bool,
}