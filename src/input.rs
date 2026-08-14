use crate::logger::log_to_file;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SetCursorPos;

/// Simulates a left mouse click at specific global screen coordinates
pub unsafe fn click_at_screen_pos(screen_x: i32, screen_y: i32) {
    log_to_file(&format!("Moving cursor & clicking at screen pos ({}, {})", screen_x, screen_y));

    // 1. Position the cursor over the yellow dot on screen
    SetCursorPos(screen_x, screen_y);

    // 2. Dispatch Left Click events via SendInput
    let mut inputs: [INPUT; 2] = std::mem::zeroed();

    inputs[0].r#type = INPUT_MOUSE;
    inputs[0].Anonymous.mi = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: MOUSEEVENTF_LEFTDOWN,
        time: 0,
        dwExtraInfo: 0,
    };

    inputs[1].r#type = INPUT_MOUSE;
    inputs[1].Anonymous.mi = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: MOUSEEVENTF_LEFTUP,
        time: 0,
        dwExtraInfo: 0,
    };

    SendInput(
        2,
        inputs.as_ptr(),
        std::mem::size_of::<INPUT>() as i32,
    );
}
