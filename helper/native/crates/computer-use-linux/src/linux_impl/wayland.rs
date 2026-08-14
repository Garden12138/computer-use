use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop};
use ashpd::desktop::screenshot::Screenshot;
use ashpd::desktop::PersistMode;
use computer_use_core::{BackendNames, Capabilities, Doctor, HelperError};
use serde_json::{json, Map, Value};

use super::{cmd_of, need_f64, need_str, opt_f64, parse_cmd_keys, scale_of};

pub fn dispatch(req: &Value) -> Result<Value, HelperError> {
    let cmd = cmd_of(req);
    match cmd.as_str() {
        "doctor" => Ok(doctor().to_value()),
        "screenshot" => screenshot(req.get("path").and_then(|v| v.as_str())),
        "get_screen_size" => screen_size(scale_of(req)),
        "cursor" => Err(HelperError::unsupported(
            "cursor on Wayland requires an active RemoteDesktop/libei session",
        )),
        "move" | "click" | "double_click" | "scroll" | "drag" | "type" | "key" | "hotkey" => {
            input(req, &cmd)
        }
        "wait" => super::handle_common_wait(req),
        "list_windows" | "get_active_window" | "focus_app" | "focus_window" => {
            Err(HelperError::unsupported(
                "window enumeration/focus is compositor-gated on Wayland (ext-foreign-toplevel-list); doctor.capabilities.focus_window is false until the compositor grants it",
            ))
        }
        other => Err(HelperError::unknown_command(other)),
    }
}

fn doctor() -> Doctor {
    let foreign = std::env::var("COMPUTER_USE_WAYLAND_FOREIGN_TOPLEVEL").ok().as_deref() == Some("1");
    Doctor {
        platform: "linux".into(),
        session: Some("wayland".into()),
        backend: BackendNames {
            input: "libei".into(),
            screen: "portal-pipewire".into(),
            window: if foreign {
                "foreign-toplevel".into()
            } else {
                "none".into()
            },
        },
        capabilities: Capabilities {
            screenshot: true,
            window_screenshot: false,
            r#move: true,
            click: true,
            scroll: true,
            r#type: true,
            list_windows: foreign,
            focus_window: false,
        },
        limitations: vec![
            "ScreenCast/RemoteDesktop portals may show a permission prompt.".into(),
            "v0.2 capture uses the XDG Screenshot portal (same portal family as ScreenCast); PipeWire frame grab can replace it later.".into(),
            "Input uses XDG RemoteDesktop Notify* plus ConnectToEIS when the portal provides an EIS fd.".into(),
            "list_windows/focus_window stay unsupported unless the compositor exposes ext-foreign-toplevel-list to this client.".into(),
        ],
        ready: true,
        extra: Map::new(),
    }
}

fn screen_size(scale: f64) -> Result<Value, HelperError> {
    Ok(json!({
        "width": 0,
        "height": 0,
        "scale": scale,
        "note": "Wayland has no global screen size API; use screenshot width/height",
    }))
}

fn screenshot(path: Option<&str>) -> Result<Value, HelperError> {
    let shot = async_std::task::block_on(async {
        Screenshot::request()
            .interactive(false)
            .send()
            .await
            .map_err(|e| HelperError::permission(e.to_string()))?
            .response()
            .map_err(|e| HelperError::permission(e.to_string()))
    })?;
    let out = output_path(path);
    copy_uri_to_png(shot.uri().as_str(), &out)?;
    Ok(json!({
        "path": out.to_string_lossy(),
        "scale": 1.0,
        "backend": "portal-pipewire",
    }))
}

fn copy_uri_to_png(uri: &str, dest: &PathBuf) -> Result<(), HelperError> {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
    std::fs::copy(path, dest).map_err(|e| HelperError::failed(e.to_string()))?;
    Ok(())
}

fn output_path(path: Option<&str>) -> PathBuf {
    if let Some(p) = path {
        return PathBuf::from(p);
    }
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    std::env::temp_dir().join(format!("computer-use-{ms}.png"))
}

fn input(req: &Value, cmd: &str) -> Result<Value, HelperError> {
    async_std::task::block_on(input_async(req, cmd))
}

async fn input_async(req: &Value, cmd: &str) -> Result<Value, HelperError> {
    let proxy = RemoteDesktop::new()
        .await
        .map_err(|e| HelperError::permission(e.to_string()))?;
    let session = proxy
        .create_session()
        .await
        .map_err(|e| HelperError::permission(e.to_string()))?;
    proxy
        .select_devices(
            &session,
            DeviceType::Keyboard | DeviceType::Pointer,
            None,
            PersistMode::DoNot,
        )
        .await
        .map_err(|e| HelperError::permission(e.to_string()))?;
    proxy
        .start(&session, None)
        .await
        .map_err(|e| HelperError::permission(e.to_string()))?
        .response()
        .map_err(|e| HelperError::permission(e.to_string()))?;
    let eis = proxy.connect_to_eis(&session).await.ok();

    match cmd {
        "move" => {
            let x = need_f64(req, "x")?;
            let y = need_f64(req, "y")?;
            proxy
                .notify_pointer_motion_absolute(&session, 0, x, y)
                .await
                .map_err(|e| HelperError::failed(e.to_string()))?;
        }
        "click" | "double_click" => {
            let x = need_f64(req, "x")?;
            let y = need_f64(req, "y")?;
            proxy
                .notify_pointer_motion_absolute(&session, 0, x, y)
                .await
                .map_err(|e| HelperError::failed(e.to_string()))?;
            let button: i32 = match req.get("button").and_then(|v| v.as_str()).unwrap_or("left") {
                "right" => 0x112,
                "middle" => 0x113,
                _ => 0x110,
            };
            let times = if cmd == "double_click" { 2 } else { 1 };
            for _ in 0..times {
                proxy
                    .notify_pointer_button(&session, button, KeyState::Pressed)
                    .await
                    .map_err(|e| HelperError::failed(e.to_string()))?;
                proxy
                    .notify_pointer_button(&session, button, KeyState::Released)
                    .await
                    .map_err(|e| HelperError::failed(e.to_string()))?;
            }
        }
        "scroll" => {
            proxy
                .notify_pointer_axis(&session, opt_f64(req, "delta_x", 0.0), opt_f64(req, "delta_y", 0.0), true)
                .await
                .map_err(|e| HelperError::failed(e.to_string()))?;
        }
        "type" => {
            let text = need_str(req, "text")?;
            for ch in text.chars() {
                let code = ch as i32;
                proxy
                    .notify_keyboard_keysym(&session, code, KeyState::Pressed)
                    .await
                    .map_err(|e| HelperError::failed(e.to_string()))?;
                proxy
                    .notify_keyboard_keysym(&session, code, KeyState::Released)
                    .await
                    .map_err(|e| HelperError::failed(e.to_string()))?;
            }
        }
        "key" | "hotkey" => {
            let keys = parse_cmd_keys(req);
            for name in &keys {
                let ks = wayland_keysym(name).ok_or_else(|| HelperError::failed(format!("unknown key {name}")))?;
                proxy
                    .notify_keyboard_keysym(&session, ks, KeyState::Pressed)
                    .await
                    .map_err(|e| HelperError::failed(e.to_string()))?;
            }
            for name in keys.iter().rev() {
                if let Some(ks) = wayland_keysym(name) {
                    let _ = proxy.notify_keyboard_keysym(&session, ks, KeyState::Released).await;
                }
            }
        }
        "drag" => {
            let x = need_f64(req, "x")?;
            let y = need_f64(req, "y")?;
            let ex = need_f64(req, "end_x")?;
            let ey = need_f64(req, "end_y")?;
            proxy.notify_pointer_motion_absolute(&session, 0, x, y).await.map_err(|e| HelperError::failed(e.to_string()))?;
            proxy.notify_pointer_button(&session, 0x110, KeyState::Pressed).await.map_err(|e| HelperError::failed(e.to_string()))?;
            proxy.notify_pointer_motion_absolute(&session, 0, ex, ey).await.map_err(|e| HelperError::failed(e.to_string()))?;
            proxy.notify_pointer_button(&session, 0x110, KeyState::Released).await.map_err(|e| HelperError::failed(e.to_string()))?;
        }
        _ => {}
    }
    Ok(json!({ "backend": "libei", "eis": eis.is_some() }))
}

fn wayland_keysym(name: &str) -> Option<i32> {
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "cmd" | "super" | "win" | "meta" => 0xffeb,
        "ctrl" | "control" => 0xffe3,
        "alt" => 0xffe9,
        "shift" => 0xffe1,
        "enter" | "return" => 0xff0d,
        "esc" | "escape" => 0xff1b,
        "tab" => 0xff09,
        "space" => 0x0020,
        other if other.len() == 1 => other.chars().next()? as i32,
        _ => return None,
    })
}
