use std::io::{self, BufRead, Write};

use serde_json::Value;

use super::{error_value, request_id, HelperError};

pub fn run_stdio<F>(mut handle: F)
where
    F: FnMut(&Value) -> Result<Value, HelperError>,
{
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                let payload = HelperError::invalid_json("request is not a JSON object").to_json("unknown");
                let _ = writeln!(stdout, "{payload}");
                let _ = stdout.flush();
                continue;
            }
        };
        let id = request_id(&req);
        let payload = match handle(&req) {
            Ok(data) => serde_json::json!({ "id": id, "ok": true, "data": data }),
            Err(err) => error_value(&id, &err),
        };
        let _ = writeln!(stdout, "{payload}");
        let _ = stdout.flush();
    }
}
