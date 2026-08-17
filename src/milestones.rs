use crate::globals::Image2SlippyError;
use std::fs;

pub fn write_milestone(
    chunk_id: (u32, u32),
    tag: &str,
    path: &str,
) -> Result<bool, Image2SlippyError> {
    if has_milestone(chunk_id, tag, path)? {
        return Ok(true);
    }

    let res = fs::create_dir_all(format!("{}/milestones", path));
    match res {
        Ok(_) => (),
        Err(err) => {
            return Err(Image2SlippyError::IOError(
                err,
                format!(
                    "Error creating the milestone directory: {}/milestones",
                    path
                ),
            ));
        }
    }
    let res = fs::File::create(format!(
        "{}/milestones/{}.{}.{}",
        path, chunk_id.0, chunk_id.1, tag
    ));
    match res {
        Ok(_) => Ok(true),
        Err(err) => match err.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(true),
            _ => Err(Image2SlippyError::IOError(
                err,
                format!(
                    "Error writing milestone file for chunk_id: {}, {} - {}",
                    chunk_id.0, chunk_id.1, tag
                ),
            )),
        },
    }
}

pub fn has_milestone(
    chunk_id: (u32, u32),
    tag: &str,
    path: &str,
) -> Result<bool, Image2SlippyError> {
    let res = fs::exists(format!(
        "{}/milestones/{}.{}.{}",
        path, chunk_id.0, chunk_id.1, tag
    ));
    match res {
        Ok(true) => Ok(true),
        Ok(false) => Ok(false),
        Err(error) => Err(Image2SlippyError::IOError(
            error,
            format!(
                "Error looking up milestone file for chunk_id {}, {} - {}",
                chunk_id.0, chunk_id.1, tag
            ),
        )),
    }
}
