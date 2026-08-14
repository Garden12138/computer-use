use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F12, VK_HOME,
    VK_LWIN, VK_LEFT, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB,
    VK_UP,
};

pub fn key_vk(name: &str) -> Option<VIRTUAL_KEY> {
    let n = name.trim().to_ascii_lowercase();
    let mapped = match n.as_str() {
        "cmd" | "ctrl" | "control" => VK_CONTROL,
        "alt" | "option" => VK_MENU,
        "shift" => VK_SHIFT,
        "win" | "super" | "meta" => VK_LWIN,
        "enter" | "return" => VK_RETURN,
        "esc" | "escape" => VK_ESCAPE,
        "tab" => VK_TAB,
        "space" => VK_SPACE,
        "backspace" => VK_BACK,
        "delete" | "del" => VK_DELETE,
        "up" => VK_UP,
        "down" => VK_DOWN,
        "left" => VK_LEFT,
        "right" => VK_RIGHT,
        "home" => VK_HOME,
        "end" => VK_END,
        "pageup" | "page_up" => VK_PRIOR,
        "pagedown" | "page_down" => VK_NEXT,
        other if other.len() == 1 => {
            let c = other.chars().next()?.to_ascii_uppercase() as u16;
            return Some(VIRTUAL_KEY(c));
        }
        other if other.starts_with('f') => {
            let n: u16 = other[1..].parse().ok()?;
            if (1..=12).contains(&n) {
                return Some(VIRTUAL_KEY(VK_F1.0 + (n - 1)));
            }
            return None;
        }
        _ => return None,
    };
    Some(mapped)
}

#[allow(dead_code)]
fn _f12() -> VIRTUAL_KEY {
    VK_F12
}
