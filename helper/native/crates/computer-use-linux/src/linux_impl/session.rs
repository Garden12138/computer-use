use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    X11,
    Wayland,
    Unknown,
}

pub fn detect_session() -> Session {
    let xtype = env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if xtype == "wayland" {
        return Session::Wayland;
    }
    if xtype == "x11" {
        return Session::X11;
    }
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return Session::Wayland;
    }
    if env::var("DISPLAY").is_ok() {
        return Session::X11;
    }
    Session::Unknown
}
