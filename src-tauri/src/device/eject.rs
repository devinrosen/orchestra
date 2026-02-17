use std::process::Command;
use orchestra_core::error::AppError;

/// Eject a volume by its mount path using macOS `diskutil`.
/// Uses `diskutil eject` which cleanly unmounts and powers down the drive.
pub fn eject_volume(mount_path: &str) -> Result<(), AppError> {
    let output = Command::new("diskutil")
        .args(["eject", mount_path])
        .output()
        .map_err(|e| AppError::General(format!("Failed to run diskutil: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("Failed to eject device: {}", stderr.trim())));
    }

    Ok(())
}
