use std::fs;

pub fn write_milestone(chunk_id: (u32, u32), tag: &str, path: &str) -> bool {
    if has_milestone(chunk_id, tag, path) {
        return true;
    }

    fs::create_dir(format!("{}/milestones", path)).unwrap_or_default();
    let res = fs::File::create(format!(
        "{}/milestones/{}.{}.{}",
        path, chunk_id.0, chunk_id.1, tag
    ));
    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn has_milestone(chunk_id: (u32, u32), tag: &str, path: &str) -> bool {
    let res = fs::exists(format!(
        "{}/milestones/{}.{}.{}",
        path, chunk_id.0, chunk_id.1, tag
    ));
    match res {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}
