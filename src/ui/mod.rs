//! Terminal user interface, table rendering, and color styling helpers.

pub mod table;

pub use table::{colored_cell, create_table, print_json_or_table, status_cell};

/// Opens a web URL in the user's default desktop browser across macOS, Linux, and Windows.
///
/// # Arguments
///
/// * `url` - The URL to open.
pub fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }
}
