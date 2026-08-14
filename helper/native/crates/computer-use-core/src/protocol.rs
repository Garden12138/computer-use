use serde_json::{json, Value};

use super::HelperError;

pub const BROWSER_RECIPES: &[&str] = &[
    "browser_open_profile",
    "browser_open_url",
    "browser_save_page",
];

pub fn is_browser_recipe(cmd: &str) -> bool {
    BROWSER_RECIPES.contains(&cmd)
}

pub fn request_id(req: &Value) -> String {
    req.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string()
}

pub fn request_cmd(req: &Value) -> String {
    req.get("cmd")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn success_value(id: &str, data: Value) -> Value {
    json!({ "id": id, "ok": true, "data": data })
}

pub fn error_value(id: &str, err: &HelperError) -> Value {
    err.to_json(id)
}

pub fn require_f64(req: &Value, key: &str) -> Result<f64, HelperError> {
    match req.get(key) {
        Some(Value::Number(n)) => n
            .as_f64()
            .ok_or_else(|| HelperError::missing(key)),
        Some(Value::String(s)) => s
            .parse::<f64>()
            .map_err(|_| HelperError::missing(key)),
        _ => Err(HelperError::missing(key)),
    }
}

pub fn optional_f64(req: &Value, key: &str, default: f64) -> f64 {
    require_f64(req, key).unwrap_or(default)
}

pub fn require_str(req: &Value, key: &str) -> Result<String, HelperError> {
    req.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| HelperError::missing(key))
}

pub fn parse_keys(req: &Value) -> Vec<String> {
    let raw = req.get("keys").or_else(|| req.get("key"));
    match raw {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Some(Value::String(s)) => s
            .split(|c| c == '+' || c == ' ' || c == ',')
            .filter(|p| !p.is_empty())
            .map(|p| p.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn jsonl_success_and_unsupported() {
        let ok = success_value("abc", json!({"width": 1}));
        assert_eq!(ok["ok"], true);
        assert_eq!(ok["id"], "abc");
        let err = HelperError::unsupported("no browser recipe");
        let payload = error_value("abc", &err);
        assert_eq!(payload["error"]["code"], "unsupported");
    }

    #[test]
    fn parse_hotkeys() {
        let req = json!({"keys": ["ctrl", "l"]});
        assert_eq!(parse_keys(&req), vec!["ctrl", "l"]);
        let req = json!({"key": "cmd+l"});
        assert_eq!(parse_keys(&req), vec!["cmd", "l"]);
    }
}
