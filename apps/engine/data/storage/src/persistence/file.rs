//! File-level helpers shared by sidecar loaders.

use piramid_core::error::Result;
use std::fs;
use std::io::{BufReader, Read};

/// Read a file front to back so its pages are resident before first use.
///
/// A missing file is not an error: there is simply nothing to warm.
pub fn warm_file(path: &str) -> Result<()> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let mut reader = BufReader::new(file);
    let mut buf = vec![0u8; 4 * 1024 * 1024]; // 4MB window to fault pages
    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        std::hint::black_box(&buf[..read]);
    }
    Ok(())
}
