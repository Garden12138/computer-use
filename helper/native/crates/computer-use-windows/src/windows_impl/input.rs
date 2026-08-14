use std::thread;
use std::time::Duration;

use computer_use_core::{screenshot_to_device, HelperError};
use serde_json::{json, Value};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_HWHEEL, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use super::keys::key_vk;

pub fn cursor(scale: f64) -> Result<Value, HelperError> {
    let mut pt = POINT::default();
    unsafe {
        GetCursorPos(&mut pt).map_err(|e| HelperError::failed(e.to_string()))?;
    }
    Ok(json!({
        "x": pt.x as f64 * scale,
        "y": pt.y as f64 * scale,
        "scale": scale,
    }))
}

pub fn r#move(x: f64, y: f64, duration: f64, scale: f64) -> Result<(), HelperError> {
    let (dx, dy) = screenshot_to_device(x, y, scale)?;
    if duration > 0.0 {
        let start = cursor(scale)?;
        let sx = start["x"].as_f64().unwrap_or(0.0);
        let sy = start["y"].as_f64().unwrap_or(0.0);
        let steps = ((duration * 60.0).ceil() as i32).max(1);
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            send_abs(sx + (x - sx) * t, sy + (y - sy) * t, scale)?;
            thread::sleep(Duration::from_secs_f64(duration / steps as f64));
        }
    }
    send_abs(dx, dy, 1.0)
}

fn send_abs(x: f64, y: f64, _scale: f64) -> Result<(), HelperError> {
    let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let nx = ((x.max(0.0) / sw.max(1) as f64) * 65535.0).round() as i32;
    let ny = ((y.max(0.0) / sh.max(1) as f64) * 65535.0).round() as i32;
    mouse_input(
        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
        nx,
        ny,
        0,
    )
}

fn mouse_input(flags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS, dx: i32, dy: i32, data: i32) -> Result<(), HelperError> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data as u32,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    send(&[input])
}

fn send(inputs: &[INPUT]) -> Result<(), HelperError> {
    unsafe {
        let n = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        if n as usize != inputs.len() {
            return Err(HelperError::failed("SendInput failed"));
        }
    }
    Ok(())
}

pub fn click(x: f64, y: f64, button: &str, count: u32, duration: f64, scale: f64) -> Result<(), HelperError> {
    r#move(x, y, duration, scale)?;
    let (down, up) = match button {
        "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        "middle" => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    };
    for _ in 0..count {
        mouse_input(down, 0, 0, 0)?;
        thread::sleep(Duration::from_millis(40));
        mouse_input(up, 0, 0, 0)?;
        thread::sleep(Duration::from_millis(40));
    }
    Ok(())
}

pub fn scroll(dx: f64, dy: f64, x: Option<f64>, y: Option<f64>, scale: f64) -> Result<(), HelperError> {
    if let (Some(x), Some(y)) = (x, y) {
        r#move(x, y, 0.05, scale)?;
    }
    if dy != 0.0 {
        mouse_input(MOUSEEVENTF_WHEEL, 0, 0, dy.round() as i32)?;
    }
    if dx != 0.0 {
        mouse_input(MOUSEEVENTF_HWHEEL, 0, 0, dx.round() as i32)?;
    }
    Ok(())
}

pub fn drag(x: f64, y: f64, end_x: f64, end_y: f64, duration: f64, scale: f64) -> Result<(), HelperError> {
    r#move(x, y, 0.0, scale)?;
    mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0)?;
    r#move(end_x, end_y, duration.max(0.05), scale)?;
    mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0)?;
    Ok(())
}

pub fn type_text(text: &str, interval: f64) -> Result<(), HelperError> {
    for ch in text.chars() {
        if ch == '\n' {
            hotkey(&["enter".into()])?;
        } else {
            unicode_key(ch)?;
        }
        if interval > 0.0 {
            thread::sleep(Duration::from_secs_f64(interval));
        }
    }
    Ok(())
}

fn unicode_key(ch: char) -> Result<(), HelperError> {
    let code = ch as u16;
    let down = key_input(VIRTUAL_KEY(0), code, KEYEVENTF_UNICODE);
    let up = key_input(VIRTUAL_KEY(0), code, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
    send(&[down, up])
}

fn key_input(vk: VIRTUAL_KEY, scan: u16, flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn hotkey(keys: &[String]) -> Result<(), HelperError> {
    let vks: Vec<VIRTUAL_KEY> = keys
        .iter()
        .map(|k| key_vk(k).ok_or_else(|| HelperError::failed(format!("unknown key {k}"))))
        .collect::<Result<_, _>>()?;
    for vk in &vks {
        send(&[key_input(*vk, 0, Default::default())])?;
    }
    thread::sleep(Duration::from_millis(30));
    for vk in vks.iter().rev() {
        send(&[key_input(*vk, 0, KEYEVENTF_KEYUP)])?;
    }
    Ok(())
}
