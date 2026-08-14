mod session;
mod wayland;
mod x11;

use computer_use_core::{
    is_browser_recipe, optional_f64, parse_keys, request_cmd, require_f64, require_str, run_stdio,
    HelperError,
};
use serde_json::{json, Value};

use session::{detect_session, Session};

pub fn main() {
    run_stdio(dispatch);
}

fn dispatch(req: &Value) -> Result<Value, HelperError> {
    let cmd = request_cmd(req);
    if is_browser_recipe(&cmd) {
        return Err(HelperError::unsupported(
            "browser recipes are macOS-only in v0.2",
        ));
    }
    match detect_session() {
        Session::X11 => x11::dispatch(req),
        Session::Wayland => wayland::dispatch(req),
        Session::Unknown => Err(HelperError::failed(
            "cannot detect Linux session; set XDG_SESSION_TYPE to x11 or wayland",
        )),
    }
}

pub(crate) fn handle_common_wait(req: &Value) -> Result<Value, HelperError> {
    let secs = optional_f64(req, "seconds", 1.0);
    std::thread::sleep(std::time::Duration::from_secs_f64(secs.max(0.0)));
    Ok(json!({}))
}

pub(crate) fn scale_of(req: &Value) -> f64 {
    optional_f64(req, "scale", 1.0)
}

pub(crate) fn parse_cmd_keys(req: &Value) -> Vec<String> {
    parse_keys(req)
}

pub(crate) fn need_f64(req: &Value, key: &str) -> Result<f64, HelperError> {
    require_f64(req, key)
}

pub(crate) fn need_str(req: &Value, key: &str) -> Result<String, HelperError> {
    require_str(req, key)
}

pub(crate) fn cmd_of(req: &Value) -> String {
    request_cmd(req)
}

pub(crate) fn opt_f64(req: &Value, key: &str, default: f64) -> f64 {
    optional_f64(req, key, default)
}
