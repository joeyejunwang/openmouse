use crate::constants::{RECT_COUNT, RECT_HEIGHT, RECT_PADDING, RECT_WIDTH};
use crate::types::RECT_CACHE;
use crate::util::{rand_xo_random, rects_overlap};
use windows_sys::Win32::Foundation::RECT;

pub fn generate_random_rects(width: i32, height: i32) -> Vec<(RECT, String)> {
    use std::collections::HashSet;
    let mut rects = Vec::new();
    let mut labels: HashSet<String> = HashSet::new();
    let max_attempts = 1000;

    let letters = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
                  'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
                  'u', 'v', 'w', 'x', 'y', 'z'];
    let numbers = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

    while rects.len() < RECT_COUNT {
        let mut attempts = 0;
        let mut placed = false;

        while attempts < max_attempts && !placed {
            let max_x = (width - RECT_WIDTH).max(1);
            let max_y = (height - RECT_HEIGHT).max(1);
            let x = (rand_xo_random() % max_x as u64) as i32;
            let y = (rand_xo_random() % max_y as u64) as i32;
            let new_rect = RECT {
                left: x,
                top: y,
                right: x + RECT_WIDTH,
                bottom: y + RECT_HEIGHT,
            };

            let overlaps = rects.iter().any(|(r, _)| rects_overlap(&new_rect, r, RECT_PADDING));
            if !overlaps {
                let letter = letters[(rand_xo_random() % letters.len() as u64) as usize];
                let num = numbers[(rand_xo_random() % numbers.len() as u64) as usize];
                let label = format!("{}{}", letter, num);

                if !labels.contains(&label) {
                    labels.insert(label.clone());
                    rects.push((new_rect, label));
                    placed = true;
                }
            }
            attempts += 1;
        }

        if !placed && rects.len() < RECT_COUNT {
            break;
        }
    }

    rects
}

pub fn get_cached_rects(width: i32, height: i32) -> Vec<(RECT, String)> {
    {
        let mut cache = RECT_CACHE.write().ok().unwrap();
        if cache.is_none() {
            let rects = generate_random_rects(width, height);
            *cache = Some(rects);
        }
    }

    RECT_CACHE.read().ok().and_then(|guard| guard.clone()).unwrap_or_default()
}
