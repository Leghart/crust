use std::path::{Path, PathBuf};

pub fn path_from_chunk(file: &Path, tmpdir: String) -> PathBuf {
    let chunk_file = file
        .to_path_buf()
        .clone()
        .to_string_lossy()
        .split('/')
        .last()
        .unwrap()
        .trim()
        .to_string();
    let path = format!("{}/{}", tmpdir, chunk_file);
    PathBuf::from(path)
}
