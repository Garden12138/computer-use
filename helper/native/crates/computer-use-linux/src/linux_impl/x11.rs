use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use computer_use_core::{
    write_png_rgb, BackendNames, Capabilities, Doctor, HelperError,
};
use serde_json::{json, Map, Value};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, EventMask, ImageFormat, Window,
};
use x11rb::protocol::xtest::ConnectionExt as XTestExt;
use x11rb::rust_connection::RustConnection;

use super::{cmd_of, need_f64, need_str, opt_f64, parse_cmd_keys, scale_of};

pub fn dispatch(req: &Value) -> Result<Value, HelperError> {
    let mut backend = X11::connect()?;
    backend.dispatch(req)
}

struct X11 {
    conn: RustConnection,
    screen: usize,
    scale: f64,
}

impl X11 {
    fn connect() -> Result<Self, HelperError> {
        let (conn, screen) = RustConnection::connect(None)
            .map_err(|e| HelperError::failed(format!("X11 connect: {e}")))?;
        Ok(Self { conn, screen, scale: 1.0 })
    }

    fn root(&self) -> Window {
        self.conn.setup().roots[self.screen].root
    }

    fn dispatch(&mut self, req: &Value) -> Result<Value, HelperError> {
        self.scale = scale_of(req);
        let cmd = cmd_of(req);
        match cmd.as_str() {
            "doctor" => Ok(self.doctor().to_value()),
            "screenshot" => self.screenshot(req.get("path").and_then(|v| v.as_str()), req.get("window_id").and_then(|v| v.as_u64())),
            "get_screen_size" => Ok(self.screen_size()),
            "cursor" => self.cursor(),
            "move" => {
                self.motion(need_f64(req, "x")?, need_f64(req, "y")?)?;
                Ok(json!({}))
            }
            "click" | "double_click" => {
                let count = if cmd == "double_click" { 2 } else { 1 };
                self.click(need_f64(req, "x")?, need_f64(req, "y")?, req.get("button").and_then(|v| v.as_str()).unwrap_or("left"), count)?;
                Ok(json!({}))
            }
            "scroll" => {
                if let (Some(x), Some(y)) = (req.get("x").and_then(|v| v.as_f64()), req.get("y").and_then(|v| v.as_f64())) {
                    self.motion(x, y)?;
                }
                self.scroll(opt_f64(req, "delta_x", 0.0), opt_f64(req, "delta_y", 0.0))?;
                Ok(json!({}))
            }
            "type" => {
                self.type_text(&need_str(req, "text")?, opt_f64(req, "interval", 0.0))?;
                Ok(json!({}))
            }
            "key" | "hotkey" => {
                self.hotkey(&parse_cmd_keys(req))?;
                Ok(json!({}))
            }
            "drag" => {
                self.drag(need_f64(req, "x")?, need_f64(req, "y")?, need_f64(req, "end_x")?, need_f64(req, "end_y")?)?;
                Ok(json!({}))
            }
            "wait" => super::handle_common_wait(req),
            "list_windows" => self.list_windows(),
            "get_active_window" => self.active_window(),
            "focus_app" => {
                self.focus_app(&need_str(req, "app")?)?;
                Ok(json!({}))
            }
            "focus_window" => {
                self.focus_window(
                    req.get("app").and_then(|v| v.as_str()),
                    req.get("title").and_then(|v| v.as_str()),
                    req.get("window_id").and_then(|v| v.as_u64()),
                )?;
                Ok(json!({}))
            }
            other => Err(HelperError::unknown_command(other)),
        }
    }

    fn doctor(&self) -> Doctor {
        Doctor {
            platform: "linux".into(),
            session: Some("x11".into()),
            backend: BackendNames {
                input: "xtest".into(),
                screen: "xgetimage".into(),
                window: "ewmh".into(),
            },
            capabilities: Capabilities::all(),
            limitations: vec!["Screenshot uses XGetImage; MIT-SHM can replace it later.".into()],
            ready: true,
            extra: Map::new(),
        }
    }

    fn screen_size(&self) -> Value {
        let scr = &self.conn.setup().roots[self.screen];
        json!({
            "width": scr.width_in_pixels,
            "height": scr.height_in_pixels,
            "width_points": scr.width_in_pixels,
            "height_points": scr.height_in_pixels,
            "scale": self.scale,
        })
    }

    fn cursor(&self) -> Result<Value, HelperError> {
        let reply = self.conn.query_pointer(self.root()).map_err(xerr)?.reply().map_err(xerr)?;
        Ok(json!({ "x": reply.root_x, "y": reply.root_y, "scale": self.scale }))
    }

    fn motion(&self, x: f64, y: f64) -> Result<(), HelperError> {
        let x = (x / self.scale).round() as i16;
        let y = (y / self.scale).round() as i16;
        self.conn.xtest_fake_input(6, 0, 0, x11rb::NONE, x, y, 0).map_err(xerr)?;
        self.conn.flush().map_err(xerr)?;
        Ok(())
    }

    fn click(&self, x: f64, y: f64, button: &str, count: u32) -> Result<(), HelperError> {
        self.motion(x, y)?;
        let btn: u8 = match button {
            "middle" => 2,
            "right" => 3,
            _ => 1,
        };
        for _ in 0..count {
            self.conn.xtest_fake_input(4, btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
            self.conn.flush().map_err(xerr)?;
            thread::sleep(Duration::from_millis(40));
            self.conn.xtest_fake_input(5, btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
            self.conn.flush().map_err(xerr)?;
        }
        Ok(())
    }

    fn scroll(&self, dx: f64, dy: f64) -> Result<(), HelperError> {
        let ticks_y = (dy / 40.0).round() as i32;
        let ticks_x = (dx / 40.0).round() as i32;
        let y_btn = if ticks_y >= 0 { 4u8 } else { 5u8 };
        for _ in 0..ticks_y.abs() {
            self.conn.xtest_fake_input(4, y_btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
            self.conn.xtest_fake_input(5, y_btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        }
        let x_btn = if ticks_x >= 0 { 7u8 } else { 6u8 };
        for _ in 0..ticks_x.abs() {
            self.conn.xtest_fake_input(4, x_btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
            self.conn.xtest_fake_input(5, x_btn, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        }
        self.conn.flush().map_err(xerr)?;
        Ok(())
    }

    fn drag(&self, x: f64, y: f64, end_x: f64, end_y: f64) -> Result<(), HelperError> {
        self.motion(x, y)?;
        self.conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        self.conn.flush().map_err(xerr)?;
        self.motion(end_x, end_y)?;
        self.conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        self.conn.flush().map_err(xerr)?;
        Ok(())
    }

    fn type_text(&self, text: &str, interval: f64) -> Result<(), HelperError> {
        for ch in text.chars() {
            if ch == '\n' {
                self.hotkey(&["enter".into()])?;
            } else {
                let keysym = latin1_keysym(ch).ok_or_else(|| HelperError::failed(format!("no X11 keysym for {ch:?}")))?;
                let code = self.keysym_to_code(keysym)?;
                self.conn.xtest_fake_input(2, code, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
                self.conn.xtest_fake_input(3, code, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
                self.conn.flush().map_err(xerr)?;
            }
            if interval > 0.0 {
                thread::sleep(Duration::from_secs_f64(interval));
            }
        }
        Ok(())
    }

    fn hotkey(&self, keys: &[String]) -> Result<(), HelperError> {
        let codes: Vec<u8> = keys.iter().map(|k| self.key_code(k)).collect::<Result<_, _>>()?;
        for c in &codes {
            self.conn.xtest_fake_input(2, *c, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        }
        self.conn.flush().map_err(xerr)?;
        thread::sleep(Duration::from_millis(30));
        for c in codes.iter().rev() {
            self.conn.xtest_fake_input(3, *c, 0, x11rb::NONE, 0, 0, 0).map_err(xerr)?;
        }
        self.conn.flush().map_err(xerr)?;
        Ok(())
    }

    fn key_code(&self, name: &str) -> Result<u8, HelperError> {
        let keysym = named_keysym(name).ok_or_else(|| HelperError::failed(format!("unknown key {name}")))?;
        self.keysym_to_code(keysym)
    }

    fn keysym_to_code(&self, keysym: u32) -> Result<u8, HelperError> {
        let setup = self.conn.setup();
        let mapping = self.conn.get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1).map_err(xerr)?.reply().map_err(xerr)?;
        let per = mapping.keysyms_per_keycode as usize;
        for (i, chunk) in mapping.keysyms.chunks(per).enumerate() {
            if chunk.iter().any(|k| *k == keysym) {
                return Ok(setup.min_keycode + i as u8);
            }
        }
        Err(HelperError::failed(format!("no keycode for keysym {keysym:#x}")))
    }

    fn intern(&self, name: &str) -> Result<u32, HelperError> {
        Ok(self.conn.intern_atom(false, name.as_bytes()).map_err(xerr)?.reply().map_err(xerr)?.atom)
    }

    fn list_windows(&self) -> Result<Value, HelperError> {
        let atom = self.intern("_NET_CLIENT_LIST")?;
        let reply = self.conn.get_property(false, self.root(), atom, AtomEnum::WINDOW, 0, 8192).map_err(xerr)?.reply().map_err(xerr)?;
        let ids: Vec<u32> = reply.value32().map(|it| it.collect()).unwrap_or_default();
        let mut windows = Vec::new();
        for id in ids {
            if let Ok(info) = self.window_info(id) {
                windows.push(info);
            }
        }
        Ok(json!({ "windows": windows }))
    }

    fn window_info(&self, id: Window) -> Result<Value, HelperError> {
        let geom = self.conn.get_geometry(id).map_err(xerr)?.reply().map_err(xerr)?;
        if geom.width < 80 || geom.height < 80 {
            return Err(HelperError::failed("too small"));
        }
        let title = self.wm_name(id).unwrap_or_default();
        let app = self.wm_class(id).unwrap_or_default();
        let pid = self.wm_pid(id).unwrap_or(0);
        Ok(json!({
            "app": app,
            "title": title,
            "window_id": id,
            "pid": pid,
            "bounds": { "x": geom.x, "y": geom.y, "width": geom.width, "height": geom.height }
        }))
    }

    fn wm_name(&self, id: Window) -> Result<String, HelperError> {
        let atom = self.intern("_NET_WM_NAME")?;
        let utf8 = self.intern("UTF8_STRING")?;
        let reply = self.conn.get_property(false, id, atom, utf8, 0, 4096).map_err(xerr)?.reply().map_err(xerr)?;
        Ok(String::from_utf8_lossy(&reply.value).into_owned())
    }

    fn wm_class(&self, id: Window) -> Result<String, HelperError> {
        let reply = self.conn.get_property(false, id, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024).map_err(xerr)?.reply().map_err(xerr)?;
        let raw = String::from_utf8_lossy(&reply.value);
        Ok(raw.split('\0').find(|s| !s.is_empty()).unwrap_or("").to_string())
    }

    fn wm_pid(&self, id: Window) -> Result<u32, HelperError> {
        let atom = self.intern("_NET_WM_PID")?;
        let reply = self.conn.get_property(false, id, atom, AtomEnum::CARDINAL, 0, 1).map_err(xerr)?.reply().map_err(xerr)?;
        Ok(reply.value32().and_then(|mut it| it.next()).unwrap_or(0))
    }

    fn active_window(&self) -> Result<Value, HelperError> {
        let atom = self.intern("_NET_ACTIVE_WINDOW")?;
        let reply = self.conn.get_property(false, self.root(), atom, AtomEnum::WINDOW, 0, 1).map_err(xerr)?.reply().map_err(xerr)?;
        let id = reply.value32().and_then(|mut it| it.next()).unwrap_or(0);
        if id == 0 {
            return Ok(json!({ "app": "", "title": "", "pid": 0, "window_id": 0 }));
        }
        self.window_info(id).or_else(|_| Ok(json!({ "window_id": id })))
    }

    fn focus_app(&self, name: &str) -> Result<(), HelperError> {
        let listed = self.list_windows()?;
        let needle = name.to_ascii_lowercase();
        for w in listed["windows"].as_array().cloned().unwrap_or_default() {
            let app = w["app"].as_str().unwrap_or("").to_ascii_lowercase();
            let title = w["title"].as_str().unwrap_or("").to_ascii_lowercase();
            if app.contains(&needle) || title.contains(&needle) {
                let id = w["window_id"].as_u64().unwrap_or(0) as u32;
                return self.activate(id);
            }
        }
        Err(HelperError::failed(format!("app not running: {name}")))
    }

    fn focus_window(&self, app: Option<&str>, title: Option<&str>, window_id: Option<u64>) -> Result<(), HelperError> {
        if let Some(id) = window_id {
            return self.activate(id as u32);
        }
        if let Some(app) = app.filter(|s| !s.is_empty()) {
            self.focus_app(app)?;
        }
        if let Some(title) = title.filter(|s| !s.is_empty()) {
            let listed = self.list_windows()?;
            let needle = title.to_ascii_lowercase();
            if let Some(w) = listed["windows"].as_array().and_then(|arr| {
                arr.iter().find(|w| w["title"].as_str().unwrap_or("").to_ascii_lowercase().contains(&needle))
            }) {
                return self.activate(w["window_id"].as_u64().unwrap_or(0) as u32);
            }
            return Err(HelperError::failed(format!("window not found: {title}")));
        }
        Ok(())
    }

    fn activate(&self, id: Window) -> Result<(), HelperError> {
        let atom = self.intern("_NET_ACTIVE_WINDOW")?;
        let event = x11rb::protocol::xproto::ClientMessageEvent::new(
            32,
            id,
            atom,
            [2, 0, 0, 0, 0],
        );
        self.conn.send_event(false, self.root(), EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, event).map_err(xerr)?;
        self.conn.flush().map_err(xerr)?;
        Ok(())
    }

    fn screenshot(&self, path: Option<&str>, window_id: Option<u64>) -> Result<Value, HelperError> {
        let win = window_id.map(|id| id as u32).unwrap_or_else(|| self.root());
        let geom = self.conn.get_geometry(win).map_err(xerr)?.reply().map_err(xerr)?;
        let image = self.conn.get_image(ImageFormat::Z_PIXMAP, win, 0, 0, geom.width, geom.height, u32::MAX).map_err(xerr)?.reply().map_err(xerr)?;
        let rgb = bgra_to_rgb(&image.data, geom.width as u32, geom.height as u32);
        let out = output_path(path);
        write_png_rgb(&out, geom.width as u32, geom.height as u32, &rgb)?;
        let mut payload = json!({
            "path": out.to_string_lossy(),
            "width": geom.width,
            "height": geom.height,
            "scale": self.scale,
        });
        if let Some(id) = window_id {
            payload["window_id"] = json!(id);
            payload["origin_x"] = json!(geom.x);
            payload["origin_y"] = json!(geom.y);
            payload["bounds_width"] = json!(geom.width);
            payload["bounds_height"] = json!(geom.height);
        }
        Ok(payload)
    }
}

fn output_path(path: Option<&str>) -> PathBuf {
    if let Some(p) = path {
        return PathBuf::from(p);
    }
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    std::env::temp_dir().join(format!("computer-use-{ms}.png"))
}

fn bgra_to_rgb(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgb = vec![0u8; width as usize * height as usize * 3];
    let stride = if data.len() >= width as usize * height as usize * 4 { 4 } else { 3 };
    for y in 0..height as usize {
        for x in 0..width as usize {
            let i = (y * width as usize + x) * stride;
            let o = (y * width as usize + x) * 3;
            if stride == 4 && i + 2 < data.len() {
                rgb[o] = data[i + 2];
                rgb[o + 1] = data[i + 1];
                rgb[o + 2] = data[i];
            } else if i + 2 < data.len() {
                rgb[o] = data[i];
                rgb[o + 1] = data[i + 1];
                rgb[o + 2] = data[i + 2];
            }
        }
    }
    rgb
}

fn xerr<E: std::fmt::Display>(e: E) -> HelperError {
    HelperError::failed(e.to_string())
}

fn latin1_keysym(ch: char) -> Option<u32> {
    let c = ch as u32;
    if c < 0x100 {
        Some(c)
    } else {
        None
    }
}

fn named_keysym(name: &str) -> Option<u32> {
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "cmd" | "super" | "win" | "meta" => 0xffeb,
        "ctrl" | "control" => 0xffe3,
        "alt" | "option" => 0xffe9,
        "shift" => 0xffe1,
        "enter" | "return" => 0xff0d,
        "esc" | "escape" => 0xff1b,
        "tab" => 0xff09,
        "space" => 0x0020,
        "backspace" => 0xff08,
        "delete" => 0xffff,
        "up" => 0xff52,
        "down" => 0xff54,
        "left" => 0xff51,
        "right" => 0xff53,
        other if other.len() == 1 => other.chars().next()? as u32,
        _ => return None,
    })
}
