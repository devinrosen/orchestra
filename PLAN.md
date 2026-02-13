# Plan: Eject/Unmount Device Button

## Summary

Add an eject button to the device card so users can safely unmount a connected device directly from the app without switching to Finder or the system tray. This involves a new Rust backend command that calls macOS `diskutil eject`, a new frontend button on the `DeviceCard` component, a confirmation dialog, and appropriate state management.

## Backend Changes

### 1. New module: `src-tauri/src/device/eject.rs`

Create a new module that handles the platform-specific eject logic:

```rust
use std::process::Command;
use crate::error::AppError;

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
```

**Why `diskutil eject` instead of `diskutil unmount`?**
- `eject` both unmounts and powers down the USB device, making it safe to physically disconnect (the standard "eject" behavior users expect).
- `unmount` only unmounts the filesystem but leaves the device powered on — users might still disconnect prematurely.
- `diskutil eject` is what Finder uses under the hood.

### 2. Register the module: `src-tauri/src/device/mod.rs`

Add `pub mod eject;` to the existing module:

```rust
pub mod detect;
pub mod eject;  // NEW
pub mod sync;
```

### 3. New Tauri command: `src-tauri/src/commands/device_cmd.rs`

Add an `eject_device` command at the end of the file:

```rust
#[tauri::command]
pub async fn eject_device(
    db: tauri::State<'_, Mutex<Connection>>,
    device_id: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let device = device_repo::get_device(&conn, &device_id)?;

    let mount_path = device
        .mount_path
        .as_ref()
        .ok_or_else(|| AppError::DeviceDisconnected(device.name.clone()))?;

    if !std::path::Path::new(mount_path).exists() {
        return Err(AppError::DeviceDisconnected(device.name.clone()));
    }

    crate::device::eject::eject_volume(mount_path)?;

    // Clear the mount_path in the database since the device is now ejected
    device_repo::update_mount_path(&conn, &device_id, "")?;

    Ok(())
}
```

Key design decisions:
- Takes `device_id` (not mount_path) so the backend controls the lookup — the frontend never passes raw paths to system commands (prevents command injection).
- After successful eject, clears the `mount_path` in the database so the device shows as disconnected without requiring a re-detect.
- The device record is preserved (not deleted) so it can be reconnected later.

### 4. Register the command: `src-tauri/src/lib.rs`

Add `commands::device_cmd::eject_device` to the `generate_handler![]` macro:

```rust
commands::device_cmd::eject_device,  // NEW — after execute_device_sync
```

### 5. Error handling

Use `AppError::General` for eject failures. Adding a new variant is unnecessary complexity — the error message from `diskutil` is descriptive enough and the frontend only needs to display it as a string. All `AppError` variants serialize to strings anyway.

## Frontend Changes

### 6. New command wrapper: `src/lib/api/commands.ts`

Add the `ejectDevice` function:

```typescript
export function ejectDevice(deviceId: string): Promise<void> {
  return invoke("eject_device", { deviceId });
}
```

### 7. Device store method: `src/lib/stores/device.svelte.ts`

Add an `ejectDevice` method and tracking state to `DeviceStore`:

```typescript
ejecting = $state<string | null>(null);  // device ID currently being ejected

async ejectDevice(deviceId: string) {
    this.ejecting = deviceId;
    this.error = null;
    try {
        await commands.ejectDevice(deviceId);
        // Update the device in local state to show as disconnected
        this.devices = this.devices.map((d) =>
            d.device.id === deviceId
                ? { ...d, connected: false, device: { ...d.device, mount_path: null } }
                : d,
        );
    } catch (e) {
        this.error = String(e);
    } finally {
        this.ejecting = null;
    }
}
```

### 8. Eject button on `DeviceCard`: `src/lib/components/DeviceCard.svelte`

Add an `onEject` callback prop and an eject button to the device card.

**Props change:**
```typescript
let {
    device,
    busy = false,
    ejecting = false,  // NEW
    onConfigure,
    onSync,
    onDelete,
    onEject,           // NEW
}: {
    device: DeviceWithStatus;
    busy?: boolean;
    ejecting?: boolean;
    onConfigure: () => void;
    onSync: () => void;
    onDelete: () => void;
    onEject: () => void;
} = $props();
```

**Button placement:** Add the eject button in the `device-actions` div. It should only be visible when the device is connected:

```html
<div class="device-actions">
    <button class="secondary" onclick={onConfigure}>Configure</button>
    {#if device.connected}
        <button
            class="eject-btn"
            onclick={onEject}
            disabled={busy || ejecting}
            title="Safely eject this device"
        >
            {ejecting ? "Ejecting..." : "Eject"}
        </button>
    {/if}
    <button
        class="primary"
        onclick={onSync}
        disabled={!device.connected || device.selected_artists.length === 0 || busy}
    >
        {busy ? "In Progress..." : "Sync"}
    </button>
    <button class="danger-btn" onclick={onDelete}>Delete</button>
</div>
```

**Styling for the eject button:**
```css
.eject-btn {
    background: none;
    color: var(--text-secondary);
    border: 1px solid var(--border);
    padding: 6px 12px;
    font-size: 13px;
}

.eject-btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--text-secondary);
}

.eject-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}
```

### 9. Wire up in `DeviceSync.svelte`: `src/pages/DeviceSync.svelte`

Add a confirmation dialog and handler.

**State:**
```typescript
let ejectingDeviceId = $state<string | null>(null);
```

**Handler:**
```typescript
function handleEjectRequest(deviceId: string) {
    ejectingDeviceId = deviceId;
}

async function confirmEject() {
    if (!ejectingDeviceId) return;
    await deviceStore.ejectDevice(ejectingDeviceId);
    ejectingDeviceId = null;
}

function cancelEject() {
    ejectingDeviceId = null;
}
```

**Pass to DeviceCard:**
```html
<DeviceCard
    {device}
    busy={isBusy}
    ejecting={deviceStore.ejecting === device.device.id}
    onConfigure={() => handleConfigure(device.device.id)}
    onSync={() => handleSync(device.device.id)}
    onDelete={() => handleDelete(device.device.id)}
    onEject={() => handleEjectRequest(device.device.id)}
/>
```

**Confirmation dialog** (reuses the same overlay pattern as the register dialog):
```html
{#if ejectingDeviceId}
    {@const ejectDevice = deviceStore.devices.find((d) => d.device.id === ejectingDeviceId)}
    <div class="register-dialog-overlay" role="presentation" onclick={cancelEject}>
        <div class="register-dialog" role="dialog" onclick={(e) => e.stopPropagation()}>
            <h3>Eject Device</h3>
            <p>Are you sure you want to eject "{ejectDevice?.device.name}"?</p>
            <p class="hint">Make sure no sync is in progress before ejecting.</p>
            <div class="dialog-actions">
                <button class="secondary" onclick={cancelEject}>Cancel</button>
                <button class="primary" onclick={confirmEject}>Eject</button>
            </div>
        </div>
    </div>
{/if}
```

## Safety Considerations

1. **Don't eject during active sync**: The eject button is disabled when `busy` is true (which covers `computing_diff` and `syncing` phases). The confirmation dialog also warns the user.

2. **Confirmation prompt**: Always show a confirmation dialog before ejecting. Accidental eject during file transfer could cause data corruption.

3. **Backend validation**: The backend verifies the device exists and is currently mounted before attempting to eject. If the mount path doesn't exist, it returns `DeviceDisconnected`.

4. **No raw paths from frontend**: The frontend sends only the `device_id`; the backend resolves the mount path internally. This prevents path injection attacks.

5. **State update after eject**: After a successful eject, the device's `mount_path` is cleared in both the database and the frontend state, so it immediately shows as "Disconnected" without needing a re-detect cycle.

6. **diskutil error handling**: If `diskutil eject` fails (e.g., "resource busy"), the error message from stderr is returned to the frontend and displayed in the error banner.

## Edge Cases

- **Device already disconnected**: If the user unplugs before clicking Eject, the backend returns `DeviceDisconnected`. The frontend shows an error banner but the device state will correct itself on next detect.
- **Eject fails because files are open**: macOS `diskutil eject` will fail with "resource busy". The error propagates to the UI. No state is changed in this case.
- **Multiple volumes on same physical device**: `diskutil eject` ejects the specific volume, not the entire physical device. If the device has multiple partitions, only the addressed volume is ejected.
- **Re-connecting after eject**: When the user reconnects and clicks "Detect Devices", the detect flow will find the volume again and update the mount_path in the database (existing logic in `detect_volumes` handles this).

## Files to Modify

| File | Action |
|------|--------|
| `src-tauri/src/device/eject.rs` | **CREATE** — eject logic using `diskutil eject` |
| `src-tauri/src/device/mod.rs` | **MODIFY** — add `pub mod eject;` |
| `src-tauri/src/commands/device_cmd.rs` | **MODIFY** — add `eject_device` command |
| `src-tauri/src/lib.rs` | **MODIFY** — register `eject_device` in `generate_handler![]` |
| `src/lib/api/commands.ts` | **MODIFY** — add `ejectDevice()` wrapper |
| `src/lib/stores/device.svelte.ts` | **MODIFY** — add `ejecting` state + `ejectDevice()` method |
| `src/lib/components/DeviceCard.svelte` | **MODIFY** — add eject button + `onEject` prop |
| `src/pages/DeviceSync.svelte` | **MODIFY** — add confirmation dialog + handler |

## Platform Considerations

- This implementation is **macOS-only** (`diskutil` is macOS-specific). This is consistent with the rest of the device detection code (`detect.rs` already uses `diskutil info -plist` and reads from `/Volumes`).
- If cross-platform support is needed later, the `eject.rs` module can be extended with `#[cfg(target_os)]` blocks for Linux (`udisksctl`) and Windows (`mountvol /d` or WMI).
- No new crate dependencies are needed — `std::process::Command` is sufficient.
