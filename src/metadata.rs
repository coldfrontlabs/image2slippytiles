use crate::chunkable::ChunkableImageSource;
use serde::{Deserialize, Serialize};

pub struct ImageProcess {
    pub image: ChunkableImageSource,
    pub image_metadata: ImageMetadata,
    pub slide_metadata: Option<SlideMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlideMetadata {
    pub mpp_x: f32,
    pub mpp_y: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TileMetadata {
    pub min_zoom: u32,
    pub max_zoom: u32,
    pub bounds: [f32; 4],
    pub peak_memory: f32,
    pub image_type: String,
    pub image_metadata: ImageMetadata,
    pub slide_metadata: Option<SlideMetadata>,
}
