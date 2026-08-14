//! Shared JSONL protocol, coordinates, PNG, and stdio loop.

mod coords;
mod doctor;
mod error;
mod png_write;
mod protocol;
mod stdio;

pub use coords::{screenshot_to_device, device_to_screenshot};
pub use doctor::{BackendNames, Capabilities, Doctor};
pub use error::HelperError;
pub use png_write::write_png_rgb;
pub use protocol::{
    error_value, is_browser_recipe, optional_f64, parse_keys, request_cmd, request_id, require_f64,
    require_str, success_value, BROWSER_RECIPES,
};
pub use stdio::run_stdio;
