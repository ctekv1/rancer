//! Export UI helpers for showing save dialogs and notifications.
//!
//! Platform-specific implementations:
//! - Linux (GTK4): Uses `gtk4::FileDialog` for native GTK save dialog
//! - Windows (winit): Uses `rfd` for native Windows save dialog

use std::path::PathBuf;

/// Generate the default export filename with timestamp
pub fn default_export_filename() -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    format!("rancer_export_{}.png", timestamp)
}

/// Show a native save file dialog and return the chosen path.
/// Returns `None` if the user cancelled.
///
/// This is a placeholder — the actual implementation is platform-specific
/// and lives in the respective window backend modules.
/// Each backend calls its own native dialog directly.
pub async fn show_save_dialog() -> Option<PathBuf> {
    // This function exists for documentation purposes.
    // The actual dialogs are implemented in:
    // - window_gtk4.rs: show_gtk_save_dialog()
    // - window_winit.rs: show_rfd_save_dialog() (Windows only)
    None
}

/// Send an OS-native notification after export completes.
/// On Linux, uses `notify-send` if available.
/// On Windows, uses the `rfd` notification or falls back to console.
pub fn notify_export_result(success: bool, path: &std::path::Path, error: Option<&str>) {
    if success {
        let msg = format!("Canvas exported to {}", path.display());
        crate::logger::info(&msg);
        println!("{}", msg);
        send_os_notification(&msg);
    } else {
        let msg = format!("Export failed: {}", error.unwrap_or("unknown error"));
        crate::logger::error(&msg);
        eprintln!("{}", msg);
        send_os_notification(&msg);
    }
}

#[cfg(target_os = "linux")]
fn send_os_notification(message: &str) {
    use std::process::Command;
    let _ = Command::new("notify-send")
        .arg("Rancer")
        .arg(message)
        .spawn();
}

#[cfg(target_os = "windows")]
fn send_os_notification(_message: &str) {
    // rfd doesn't have a notification API on Windows
    // Console print + logger is sufficient
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn send_os_notification(_message: &str) {
    // Fallback: no OS notification
}
