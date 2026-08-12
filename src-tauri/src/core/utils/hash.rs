use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Calculates the complete SHA-256 digest of a file.
pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let mut file =
        File::open(file_path).with_context(|| format!("无法打开文件: {}", file_path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("读取文件失败: {}", file_path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
