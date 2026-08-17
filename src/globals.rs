use image::error;
use peak_alloc::PeakAlloc;
use std::io;

#[global_allocator]
pub static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub fn memory_check() -> (f32, f32) {
    let current_mem = PEAK_ALLOC.current_usage_as_mb();
    let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
    (current_mem, peak_mem)
}

#[derive(Debug)]
pub enum Image2SlippyError {
    IOError(io::Error, String),
    ImageError(error::ImageError),
    Image2SlippyError(String),
}

impl From<error::ImageError> for Image2SlippyError {
    fn from(error: error::ImageError) -> Self {
        Image2SlippyError::ImageError(error)
    }
}

impl From<String> for Image2SlippyError {
    fn from(error: String) -> Self {
        Image2SlippyError::Image2SlippyError(error)
    }
}

impl std::fmt::Display for Image2SlippyError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Image2SlippyError::IOError(err, context) => {
                write!(fmt, "Got \"{}\" while \"{}\"", err, context)
            }
            Image2SlippyError::ImageError(err) => write!(fmt, "{}", err),
            Image2SlippyError::Image2SlippyError(err) => write!(fmt, "{}", err),
        }
    }
}
