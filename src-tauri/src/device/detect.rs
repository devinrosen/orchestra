use std::path::Path;
use std::process::Command;

use orchestra_core::error::AppError;
use orchestra_core::models::device::DetectedVolume;

pub fn detect_usb_volumes() -> Result<Vec<DetectedVolume>, AppError> {
    let volumes_dir = Path::new("/Volumes");
    if !volumes_dir.exists() {
        return Ok(vec![]);
    }

    let mut detected = Vec::new();

    let entries = std::fs::read_dir(volumes_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip symlinks (macOS root volume is a symlink in /Volumes)
        if path.read_link().is_ok() {
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        if let Some(volume) = probe_volume(&path) {
            detected.push(volume);
        }
    }

    Ok(detected)
}

fn probe_volume(mount_path: &Path) -> Option<DetectedVolume> {
    let output = Command::new("diskutil")
        .args(["info", "-plist"])
        .arg(mount_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let plist: plist::Dictionary = plist::from_bytes(&output.stdout).ok()?;

    // Filter: must be external and removable
    let is_internal = plist
        .get("Internal")
        .and_then(|v| v.as_boolean())
        .unwrap_or(true);
    let is_removable = plist
        .get("RemovableMedia")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);
    // Also check DeviceLocation for "External"
    let device_location = plist
        .get("DeviceLocation")
        .and_then(|v| v.as_string())
        .unwrap_or("");

    // Accept if either: explicitly external, or removable and not internal
    if is_internal && device_location != "External" && !is_removable {
        return None;
    }

    let volume_uuid = plist
        .get("VolumeUUID")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())?; // Must have a VolumeUUID

    let volume_name = plist
        .get("VolumeName")
        .and_then(|v| v.as_string())
        .unwrap_or("Untitled")
        .to_string();

    let total_size = plist
        .get("TotalSize")
        .and_then(|v| v.as_unsigned_integer())
        .unwrap_or(0);

    let free_space = plist
        .get("APFSContainerFree")
        .or_else(|| plist.get("FreeSpace"))
        .and_then(|v| v.as_unsigned_integer())
        .unwrap_or(0);

    let bus_protocol = plist
        .get("BusProtocol")
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown")
        .to_string();

    Some(DetectedVolume {
        volume_uuid,
        volume_name,
        mount_path: mount_path.to_string_lossy().to_string(),
        capacity_bytes: total_size,
        free_bytes: free_space,
        bus_protocol,
        already_registered: false,
    })
}
