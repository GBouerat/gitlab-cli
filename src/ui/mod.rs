//! Terminal user interface, table rendering, and color styling helpers.

pub mod table;

pub use table::{colored_cell, create_table, print_json_or_table, status_cell};

/// Opens a web URL in the user's default desktop browser across macOS, Linux, and Windows.
///
/// If spawning the system browser launcher fails (such as in headless or SSH environments),
/// a debug trace is logged.
///
/// # Arguments
///
/// * `url` - The URL to open.
pub fn open_in_browser(url: &str) {
    let spawn_result = {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(url).spawn()
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open").arg(url).spawn()
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", url])
                .spawn()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported operating system for browser launch",
            ))
        }
    };

    if let Err(err) = spawn_result {
        tracing::debug!("Failed to launch browser for '{url}': {err}");
    }
}
