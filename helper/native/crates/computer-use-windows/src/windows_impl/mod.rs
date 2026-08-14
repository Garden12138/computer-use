mod capture;
mod dpi;
mod input;
mod keys;
mod windows_api;

use computer_use_core::{
    is_browser_recipe, optional_f64, parse_keys, request_cmd, require_f64, require_str, run_stdio,
    BackendNames, Capabilities, Doctor, HelperError,
};
use serde_json::{json, Map, Value};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

pub fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    dpi::enable_per_monitor_v2();
    run_stdio(dispatch);
}

fn dispatch(req: &Value) -> Result<Value, HelperError> {
    let cmd = request_cmd(req);
    if is_browser_recipe(&cmd) {
        return Err(HelperError::unsupported(
            "browser recipes are macOS-only in v0.2",
        ));
    }
    let scale = optional_f64(req, "scale", dpi::scale());
    match cmd.as_str() {
        "doctor" => Ok(doctor().to_value()),
        "screenshot" => capture::screenshot(req.get("path").and_then(|v| v.as_str()), req.get("window_id").and_then(|v| v.as_u64()).map(|id| HWND(id as *mut _)), scale),
        "get_screen_size" => Ok(capture::screen_size(scale)),
        "cursor" => input::cursor(scale),
        "move" => {
            input::r#move(require_f64(req, "x")?, require_f64(req, "y")?, optional_f64(req, "duration", 0.0), scale)?;
            Ok(json!({}))
        }
        "click" | "double_click" => {
            let count = if cmd == "double_click" { 2 } else { optional_f64(req, "count", 1.0) as u32 };
            input::click(
                require_f64(req, "x")?,
                require_f64(req, "y")?,
                req.get("button").and_then(|v| v.as_str()).unwrap_or("left"),
                count.max(1),
                optional_f64(req, "duration", 0.0),
                scale,
            )?;
            Ok(json!({}))
        }
        "scroll" => {
            input::scroll(
                optional_f64(req, "delta_x", 0.0),
                optional_f64(req, "delta_y", 0.0),
                req.get("x").and_then(|v| v.as_f64()),
                req.get("y").and_then(|v| v.as_f64()),
                scale,
            )?;
            Ok(json!({}))
        }
        "type" => {
            input::type_text(&require_str(req, "text")?, optional_f64(req, "interval", 0.0))?;
            Ok(json!({}))
        }
        "key" | "hotkey" => {
            input::hotkey(&parse_keys(req))?;
            Ok(json!({}))
        }
        "drag" => {
            input::drag(
                require_f64(req, "x")?,
                require_f64(req, "y")?,
                require_f64(req, "end_x")?,
                require_f64(req, "end_y")?,
                optional_f64(req, "duration", 0.2),
                scale,
            )?;
            Ok(json!({}))
        }
        "wait" => {
            let secs = optional_f64(req, "seconds", 1.0);
            std::thread::sleep(std::time::Duration::from_secs_f64(secs.max(0.0)));
            Ok(json!({}))
        }
        "list_windows" => windows_api::list_windows(),
        "get_active_window" => windows_api::active_window(),
        "focus_app" => {
            windows_api::focus_app(&require_str(req, "app")?)?;
            Ok(json!({}))
        }
        "focus_window" => {
            let hwnd = req.get("window_id").and_then(|v| v.as_u64()).map(|id| HWND(id as *mut _));
            windows_api::focus_window(
                req.get("app").and_then(|v| v.as_str()),
                req.get("title").and_then(|v| v.as_str()),
                hwnd,
            )?;
            Ok(json!({}))
        }
        "" => Err(HelperError::unknown_command("")),
        other => Err(HelperError::unknown_command(other)),
    }
}

fn doctor() -> Doctor {
    Doctor {
        platform: "windows".into(),
        session: None,
        backend: BackendNames {
            input: "win32-sendinput".into(),
            screen: "windows-graphics-capture".into(),
            window: "win32-user32".into(),
        },
        capabilities: Capabilities::all(),
        limitations: vec![
            "SetForegroundWindow may return focus_denied because Windows restricts stealing focus.".into(),
        ],
        ready: true,
        extra: Map::new(),
    }
}
