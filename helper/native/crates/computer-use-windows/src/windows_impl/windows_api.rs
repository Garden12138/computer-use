use computer_use_core::HelperError;
use serde_json::{json, Value};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, MAX_PATH};
use windows::Win32::System::ProcessStatus::K32GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

struct EnumState {
    windows: Vec<Value>,
}

pub fn list_windows() -> Result<Value, HelperError> {
    let mut state = EnumState { windows: Vec::new() };
    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(&mut state as *mut _ as isize))
            .map_err(|e| HelperError::failed(e.to_string()))?;
    }
    Ok(json!({ "windows": state.windows }))
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    let mut rect = windows::Win32::Foundation::RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return BOOL(1);
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width < 80 || height < 80 {
        return BOOL(1);
    }
    let title = window_title(hwnd);
    if title.is_empty() {
        return BOOL(1);
    }
    let (pid, app) = process_name(hwnd);
    state.windows.push(json!({
        "app": app,
        "title": title,
        "window_id": hwnd.0 as u64,
        "pid": pid,
        "bounds": {
            "x": rect.left,
            "y": rect.top,
            "width": width,
            "height": height,
        }
    }));
    BOOL(1)
}

fn window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

fn process_name(hwnd: HWND) -> (u32, String) {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
    let Ok(handle) = handle else {
        return (pid, String::new());
    };
    let mut buf = [0u16; MAX_PATH as usize];
    let n = unsafe { K32GetModuleBaseNameW(handle, None, &mut buf) };
    unsafe { let _ = CloseHandle(handle); }
    let name = if n > 0 {
        String::from_utf16_lossy(&buf[..n as usize])
    } else {
        String::new()
    };
    (pid, name)
}

pub fn active_window() -> Result<Value, HelperError> {
    let hwnd = unsafe { GetForegroundWindow() };
    let (pid, app) = process_name(hwnd);
    Ok(json!({
        "app": app,
        "title": window_title(hwnd),
        "pid": pid,
        "window_id": hwnd.0 as u64,
    }))
}

pub fn focus_app(name: &str) -> Result<(), HelperError> {
    let listed = list_windows()?;
    let windows = listed["windows"].as_array().cloned().unwrap_or_default();
    let needle = name.to_ascii_lowercase();
    for w in windows {
        let app = w["app"].as_str().unwrap_or("").to_ascii_lowercase();
        let title = w["title"].as_str().unwrap_or("").to_ascii_lowercase();
        if app.contains(&needle) || title.contains(&needle) {
            let id = w["window_id"].as_u64().unwrap_or(0);
            return focus_hwnd(HWND(id as *mut _));
        }
    }
    Err(HelperError::failed(format!("app not running: {name}")))
}

pub fn focus_window(app: Option<&str>, title: Option<&str>, hwnd: Option<HWND>) -> Result<(), HelperError> {
    if let Some(h) = hwnd {
        return focus_hwnd(h);
    }
    if let Some(app) = app.filter(|s| !s.is_empty()) {
        focus_app(app)?;
    }
    if let Some(title) = title.filter(|s| !s.is_empty()) {
        let listed = list_windows()?;
        let needle = title.to_ascii_lowercase();
        if let Some(w) = listed["windows"].as_array().and_then(|arr| {
            arr.iter().find(|w| {
                w["title"].as_str().unwrap_or("").to_ascii_lowercase().contains(&needle)
            })
        }) {
            let id = w["window_id"].as_u64().unwrap_or(0);
            return focus_hwnd(HWND(id as *mut _));
        }
        return Err(HelperError::failed(format!("window not found: {title}")));
    }
    Ok(())
}

fn focus_hwnd(hwnd: HWND) -> Result<(), HelperError> {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        if !SetForegroundWindow(hwnd).as_bool() {
            return Err(HelperError::focus_denied(
                "SetForegroundWindow refused (Windows foreground lock)",
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn _pwstr(_p: PWSTR) {}
