//! Export UI helpers for showing save dialogs and notifications.

use std::path::PathBuf;

/// Generate the default export filename with timestamp
pub fn default_export_filename() -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    format!("rancer_export_{}.png", timestamp)
}

/// Show a native save file dialog and return the chosen path.
/// Returns `None` if the user cancelled.
pub fn show_save_dialog() -> Option<PathBuf> {
    let filename = default_export_filename();
    rfd::FileDialog::new()
        .set_file_name(&filename)
        .add_filter("PNG Image", &["png"])
        .save_file()
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
