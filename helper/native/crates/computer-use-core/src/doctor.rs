use serde::Serialize;
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Serialize)]
pub struct BackendNames {
    pub input: String,
    pub screen: String,
    pub window: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Capabilities {
    pub screenshot: bool,
    pub window_screenshot: bool,
    pub r#move: bool,
    pub click: bool,
    pub scroll: bool,
    pub r#type: bool,
    pub list_windows: bool,
    pub focus_window: bool,
}

impl Capabilities {
    pub fn all() -> Self {
        Self {
            screenshot: true,
            window_screenshot: true,
            r#move: true,
            click: true,
            scroll: true,
            r#type: true,
            list_windows: true,
            focus_window: true,
        }
    }

    pub fn to_map(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("screenshot".into(), json!(self.screenshot));
        m.insert("window_screenshot".into(), json!(self.window_screenshot));
        m.insert("move".into(), json!(self.r#move));
        m.insert("click".into(), json!(self.click));
        m.insert("scroll".into(), json!(self.scroll));
        m.insert("type".into(), json!(self.r#type));
        m.insert("list_windows".into(), json!(self.list_windows));
        m.insert("focus_window".into(), json!(self.focus_window));
        m
    }
}

#[derive(Debug, Clone)]
pub struct Doctor {
    pub platform: String,
    pub session: Option<String>,
    pub backend: BackendNames,
    pub capabilities: Capabilities,
    pub limitations: Vec<String>,
    pub ready: bool,
    pub extra: Map<String, Value>,
}

impl Doctor {
    pub fn to_value(&self) -> Value {
        let mut data = Map::new();
        data.insert("platform".into(), json!(self.platform));
        if let Some(session) = &self.session {
            data.insert("session".into(), json!(session));
        }
        data.insert(
            "backend".into(),
            json!({
                "input": self.backend.input,
                "screen": self.backend.screen,
                "window": self.backend.window,
            }),
        );
        data.insert("capabilities".into(), Value::Object(self.capabilities.to_map()));
        data.insert("limitations".into(), json!(self.limitations));
        data.insert("ready".into(), json!(self.ready));
        for (k, v) in &self.extra {
            data.insert(k.clone(), v.clone());
        }
        Value::Object(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_has_required_keys() {
        let doc = Doctor {
            platform: "windows".into(),
            session: None,
            backend: BackendNames {
                input: "win32-sendinput".into(),
                screen: "windows-graphics-capture".into(),
                window: "win32-user32".into(),
            },
            capabilities: Capabilities::all(),
            limitations: vec![],
            ready: true,
            extra: Map::new(),
        };
        let v = doc.to_value();
        assert_eq!(v["platform"], "windows");
        assert_eq!(v["backend"]["input"], "win32-sendinput");
        assert_eq!(v["capabilities"]["focus_window"], true);
        assert!(v["limitations"].is_array());
        assert_eq!(v["ready"], true);
    }
}
