pub mod frontmatter;
pub mod jsonl;

use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::error::EccResult;

/// Minimal filesystem abstraction for testability.
/// RealFileSystem delegates to std::fs.
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn read_to_string(path: &Path) -> EccResult<String> {
        Ok(fs::read_to_string(path)?)
    }

    pub fn read_dir_entries(path: &Path) -> EccResult<Vec<PathBuf>> {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        entries.sort();
        Ok(entries)
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn is_dir(path: &Path) -> bool {
        path.is_dir()
    }

    pub fn file_size(path: &Path) -> EccResult<u64> {
        Ok(fs::metadata(path)?.len())
    }

    pub fn open_buffered(path: &Path) -> EccResult<Box<dyn BufRead>> {
        let file = fs::File::open(path)?;
        Ok(Box::new(std::io::BufReader::new(file)))
    }
}
