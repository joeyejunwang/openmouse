use windows_sys::Win32::Foundation::RECT;

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub fn rects_overlap(a: &RECT, b: &RECT, padding: i32) -> bool {
    !(a.right + padding < b.left || a.left > b.right + padding ||
      a.bottom + padding < b.top || a.top > b.bottom + padding)
}

// Simple xorshift RNG for deterministic randomness
static mut RNG_STATE: u64 = 0x123456789ABCDEF0u64;

pub fn rand_xo_random() -> u64 {
    unsafe {
        let mut x = RNG_STATE;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        RNG_STATE = x;
        x
    }
}
