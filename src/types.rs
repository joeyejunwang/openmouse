use std::sync::RwLock;
use windows_sys::Win32::Foundation::RECT;

use crate::constants::{DOT_PADDING, OVERLAY_X, OVERLAY_Y};

pub static RECT_CACHE: RwLock<Option<Vec<(RECT, String)>>> = RwLock::new(None);

pub fn point_in_rect(x: i32, y: i32, rect: &RECT) -> bool {
    x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
}

pub fn get_rect_at_point(x: i32, y: i32) -> Option<(RECT, String)> {
    if let Ok(cache) = RECT_CACHE.read() {
        if let Some(ref rects) = *cache {
            return rects.iter().find(|(rect, _)| point_in_rect(x, y, rect)).cloned();
        }
    }
    None
}

pub fn screen_pos_for_rect(rect: &RECT) -> (i32, i32) {
    (OVERLAY_X + rect.left + DOT_PADDING, OVERLAY_Y + rect.top + DOT_PADDING)
}
