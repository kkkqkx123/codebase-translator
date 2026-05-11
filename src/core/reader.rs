use std::path::Path;
use tracing::debug;

use crate::core::error::Result;
use crate::core::models::File;
use crate::encoding::{Detector, Encoder};

pub fn read_text_file(path: &Path) -> Result<File> {
    debug!(file = %path.display(), "Reading text file with encoding detection");

    let content_bytes = std::fs::read(path)?;

    let detector = Detector::default();
    let encoding_result = detector.detect_bytes(&content_bytes)?;
    let encoding = encoding_result.encoding;

    let encoder = Encoder::default();
    let utf8_content = encoder.to_utf8(&content_bytes, &encoding)?;

    Ok(File::new(path.to_path_buf(), utf8_content.into_bytes(), encoding))
}
