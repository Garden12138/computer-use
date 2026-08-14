use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct HelperError {
    pub code: &'static str,
    pub message: String,
}

impl HelperError {
    pub fn invalid_json(msg: impl Into<String>) -> Self {
        Self { code: "invalid_json", message: msg.into() }
    }
    pub fn unknown_command(cmd: impl Into<String>) -> Self {
        Self { code: "unknown_command", message: format!("unknown command: {}", cmd.into()) }
    }
    pub fn missing(name: impl Into<String>) -> Self {
        Self { code: "invalid_argument", message: format!("missing argument: {}", name.into()) }
    }
    pub fn permission(msg: impl Into<String>) -> Self {
        Self { code: "permission_denied", message: msg.into() }
    }
    pub fn failed(msg: impl Into<String>) -> Self {
        Self { code: "failed", message: msg.into() }
    }
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self { code: "unsupported", message: msg.into() }
    }
    pub fn focus_denied(msg: impl Into<String>) -> Self {
        Self { code: "focus_denied", message: msg.into() }
    }

    pub fn to_json(&self, id: &str) -> Value {
        json!({
            "id": id,
            "ok": false,
            "error": { "code": self.code, "message": self.message }
        })
    }
}
