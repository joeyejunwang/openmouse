use crate::constants::{
    DOT_PADDING, DOT_SIZE, HOTKEY_ID_EXIT, OVERLAY_HEIGHT, OVERLAY_WIDTH,
    OVERLAY_X, OVERLAY_Y, TIMER_ID_SCREENSHOT, WINDOW_ALPHA,
};
use crate::input::click_at_screen_pos;
use crate::logger::log_to_file;
use crate::rect::get_cached_rects;
use crate::screenshot::take_screenshot;
use crate::types::{get_rect_at_point, screen_pos_for_rect};
use crate::util::rgb;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, DrawTextA, Ellipse, EndPaint,
    FillRect, GetStockObject, SelectObject, SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE,
    DT_VCENTER, NULL_PEN, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_CONTROL, VK_CONTROL, VK_F12,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GetMessageA,
    KillTimer, PostQuitMessage, RegisterClassA, SetLayeredWindowAttributes,
    SetTimer, ShowWindow, CS_HREDRAW, CS_VREDRAW, LWA_ALPHA, LWA_COLORKEY, MSG, SW_HIDE,
    SW_SHOW, WM_DESTROY, WM_HOTKEY, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_PAINT, WM_TIMER, WNDCLASSA,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_POPUP,
};

// Track if overlay is currently visible
static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(true);
// Track if Ctrl key is currently held
static CTRL_HELD: AtomicBool = AtomicBool::new(false);

fn create_overlay_window() -> Option<HWND> {
    unsafe {
        let instance = GetModuleHandleA(null_mut());
        let class_name = b"RustOverlayRectsClass\0";

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassA(&wnd_class);

        let ex_style = WS_EX_TOPMOST | WS_EX_LAYERED;

        let hwnd = CreateWindowExA(
            ex_style,
            class_name.as_ptr(),
            b"Persistent Rectangles Overlay\0".as_ptr(),
            WS_POPUP,
            OVERLAY_X, OVERLAY_Y, OVERLAY_WIDTH, OVERLAY_HEIGHT,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );

        if hwnd.is_null() {
            log_to_file("Failed to create overlay window.");
            return None;
        }

        let magenta_key = rgb(255, 0, 255);
        SetLayeredWindowAttributes(hwnd, magenta_key, WINDOW_ALPHA, LWA_COLORKEY | LWA_ALPHA);

        // Register Ctrl + F12 exit hotkey
        RegisterHotKey(hwnd, HOTKEY_ID_EXIT, MOD_CONTROL, VK_F12 as u32);

        SetTimer(hwnd, TIMER_ID_SCREENSHOT, 10_000, None);

        ShowWindow(hwnd, SW_SHOW);
        log_to_file("Overlay window shown. Click rectangles to click dots.");

        Some(hwnd)
    }
}

fn run_message_loop() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageA(&mut msg, null_mut(), 0, 0) > 0 {
            DispatchMessageA(&mut msg);
        }
    }
}

pub fn init_overlay() {
    log_to_file("Application starting overlay...");

    if create_overlay_window().is_some() {
        run_message_loop();
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let hotkey_id = wparam as i32;

            if hotkey_id == HOTKEY_ID_EXIT {
                log_to_file("Ctrl+F12 pressed! Closing application.");
                DestroyWindow(hwnd);
            }
            0
        }
        WM_TIMER => {
            take_screenshot();
            0
        }
        WM_KEYDOWN => {
            if wparam == VK_CONTROL as WPARAM && !CTRL_HELD.load(Ordering::Relaxed) {
                CTRL_HELD.store(true, Ordering::Relaxed);
                
                let new_state = !OVERLAY_VISIBLE.load(Ordering::Relaxed);
                OVERLAY_VISIBLE.store(new_state, Ordering::Relaxed);
                
                if new_state {
                    unsafe { ShowWindow(hwnd, SW_SHOW); }
                    log_to_file("Ctrl pressed - showing overlay");
                } else {
                    unsafe { ShowWindow(hwnd, SW_HIDE); }
                    log_to_file("Ctrl pressed - hiding overlay");
                }
            }
            0
        }
        WM_KEYUP => {
            if wparam == VK_CONTROL as WPARAM {
                CTRL_HELD.store(false, Ordering::Relaxed);
            }
            0
        }
        WM_LBUTTONDOWN => {
            // Get click position from lparam (LOWORD=x, HIWORD=y)
            let x = (lparam & 0xFFFF) as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i32;

            if let Some((rect, label)) = get_rect_at_point(x, y) {
                let (screen_x, screen_y) = screen_pos_for_rect(&rect);
                log_to_file(&format!("Clicked rect '{}' at ({}, {}), clicking dot at ({}, {})",
                    label, x, y, screen_x, screen_y));
                click_at_screen_pos(screen_x, screen_y);

                // Hide overlay after click
                OVERLAY_VISIBLE.store(false, Ordering::Relaxed);
                unsafe { ShowWindow(hwnd, SW_HIDE); }
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            // Background fill
            let magenta_brush = unsafe { CreateSolidBrush(rgb(255, 0, 255)) };
            let bg_rect = RECT { left: 0, top: 0, right: OVERLAY_WIDTH, bottom: OVERLAY_HEIGHT };
            unsafe { FillRect(hdc, &bg_rect, magenta_brush); };
            unsafe { DeleteObject(magenta_brush); };

            // Render red boxes with labels
            let red_brush = unsafe { CreateSolidBrush(rgb(255, 0, 0)) };
            let yellow_brush = unsafe { CreateSolidBrush(rgb(255, 255, 0)) };
            let null_pen = unsafe { GetStockObject(NULL_PEN) };

            let rectangles = get_cached_rects(OVERLAY_WIDTH, OVERLAY_HEIGHT);

            unsafe { SetBkMode(hdc, TRANSPARENT as i32); };
            unsafe { SetTextColor(hdc, rgb(255, 255, 255)); };

            for (_, (mut rect, label)) in rectangles.into_iter().enumerate() {
                unsafe { FillRect(hdc, &rect, red_brush); };

                // Draw yellow dot
                let dot_x = rect.left + DOT_PADDING;
                let dot_y = rect.top + DOT_PADDING;

                let old_brush = unsafe { SelectObject(hdc, yellow_brush) };
                let old_pen = unsafe { SelectObject(hdc, null_pen) };

                unsafe { Ellipse(hdc, dot_x, dot_y, dot_x + DOT_SIZE, dot_y + DOT_SIZE); };

                unsafe { SelectObject(hdc, old_pen); };
                unsafe { SelectObject(hdc, old_brush); };

                // Render label
                let label_str = format!("{}\0", label);
                unsafe {
                    DrawTextA(
                        hdc,
                        label_str.as_ptr(),
                        -1,
                        &mut rect as *mut RECT,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                }
            }

            unsafe { DeleteObject(yellow_brush); };
            unsafe { DeleteObject(red_brush); };
            unsafe { EndPaint(hwnd, &ps); };
            0
        }
        WM_DESTROY => {
            unsafe {
                UnregisterHotKey(hwnd, HOTKEY_ID_EXIT);
                KillTimer(hwnd, TIMER_ID_SCREENSHOT);
            };
            log_to_file("WM_DESTROY received. Exiting.");
            unsafe { PostQuitMessage(0); };
            0
        }
        _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
    }
}
