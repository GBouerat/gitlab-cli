//! Terminal table creation and formatting using `comfy-table`.
//!
//! Enforces proper alignment by applying colors and bold styles directly to
//! [`comfy_table::Cell`] objects rather than embedding raw ANSI escape sequences.

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use serde::Serialize;

/// Creates a standardized, border-styled [`Table`] configured for responsive terminal output.
///
/// # Arguments
///
/// * `headers` - Header row column titles.
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(*h).add_attribute(Attribute::Bold).fg(Color::Cyan))
        .collect();

    table.set_header(header_cells);
    table
}

/// Creates a [`Cell`] with specified foreground color and optional bold styling.
///
/// # Arguments
///
/// * `content` - Text content of the cell.
/// * `color` - Color to apply to text.
/// * `bold` - Whether to render text in bold.
pub fn colored_cell(content: impl Into<String>, color: Color, bold: bool) -> Cell {
    let mut cell = Cell::new(content.into()).fg(color);
    if bold {
        cell = cell.add_attribute(Attribute::Bold);
    }
    cell
}

/// Generates a styled [`Cell`] with intuitive coloring mapped to a resource status string.
///
/// Examples:
/// - `success`, `opened`, `active` -> Green
/// - `failed`, `closed`, `canceled` -> Red
/// - `running`, `merged` -> Blue/Cyan
/// - `pending`, `draft`, `skipped`, `manual` -> Yellow
///
/// # Arguments
///
/// * `status` - The status identifier.
pub fn status_cell(status: &str) -> Cell {
    let normalized = status.to_lowercase();
    match normalized.as_str() {
        "opened" | "success" | "active" | "can_be_merged" => {
            colored_cell(status, Color::Green, true)
        }
        "failed" | "closed" | "canceled" | "cannot_be_merged" => {
            colored_cell(status, Color::Red, true)
        }
        "running" | "merged" => colored_cell(status, Color::Cyan, true),
        "pending" | "draft" | "skipped" | "manual" | "waiting_for_resource" => {
            colored_cell(status, Color::Yellow, true)
        }
        _ => Cell::new(status),
    }
}

/// Prints data either as serialized JSON or renders the provided table depending on `json_mode`.
///
/// # Arguments
///
/// * `data` - Serializable data payload.
/// * `table` - Formatted `comfy_table::Table` to print if not in JSON mode.
/// * `json_mode` - If `true`, prints pretty-printed JSON to stdout instead of the table.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn print_json_or_table<T: Serialize>(
    data: &T,
    table: &Table,
    json_mode: bool,
) -> anyhow::Result<()> {
    if json_mode {
        let json = serde_json::to_string_pretty(data)?;
        println!("{json}");
    } else {
        println!("{table}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table() {
        let mut table = create_table(&["ID", "Title", "Status"]);
        assert_eq!(table.column_count(), 3);
    }

    #[test]
    fn test_status_cell_colors() {
        let cell_ok = status_cell("opened");
        assert_eq!(cell_ok.content(), "opened");

        let cell_err = status_cell("failed");
        assert_eq!(cell_err.content(), "failed");
    }
}
