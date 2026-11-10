use bevy::prelude::*;
use std::collections::VecDeque;
use crate::constants::{COLS, ROWS, CELL, LANE_W, LANE_H, LANE_X, LANE_Y};

pub fn world_to_grid(x: f32, y: f32) -> Option<(usize, usize)> {
    let left = LANE_X - LANE_W / 2.0;
    let bottom = LANE_Y - LANE_H / 2.0;
    if x < left || x > left + LANE_W { return None; }
    if y < bottom || y > bottom + LANE_H { return None; }
    let col = ((x - left) / CELL).floor() as usize;
    let row = ((y - bottom) / CELL).floor() as usize;
    if col >= COLS || row >= ROWS { return None; }
    Some((row, col))
}

pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    let left = LANE_X - LANE_W / 2.0;
    let bottom = LANE_Y - LANE_H / 2.0;
    Vec2::new(
        left + col as f32 * CELL + CELL / 2.0,
        bottom + row as f32 * CELL + CELL / 2.0,
    )
}

pub fn compute_path(blocked: &[[bool; COLS]; ROWS]) -> [[Option<i32>; COLS]; ROWS] {
    let mut dist: [[Option<i32>; COLS]; ROWS] = [[None; COLS]; ROWS];
    let mut q = VecDeque::new();
    for c in 0..COLS {
        if !blocked[0][c] {
            dist[0][c] = Some(0);
            q.push_back((0, c));
        }
    }
    let dirs = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)];
    while let Some((r, c)) = q.pop_front() {
        let cur_d = dist[r][c].unwrap();
        for (dr, dc) in dirs.iter() {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize { continue; }
            let nr = nr as usize;
            let nc = nc as usize;
            if blocked[nr][nc] { continue; }
            if dist[nr][nc].is_some() { continue; }
            dist[nr][nc] = Some(cur_d + 1);
            q.push_back((nr, nc));
        }
    }
    dist
}

// Android: okraje systemovych list v PIXELECH (vlevo, nahore, vpravo, dole) pres JNI; None kdyz zatim nejsou k dispozici
#[cfg(target_os = "android")]
pub fn android_insets_px() -> Option<(i32, i32, i32, i32)> {
    use jni::objects::JObject;
    use jni::JavaVM;
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    // with_local_frame: lokalni reference se uvolni po kazdem cteni (cteme opakovane)
    let res = env.with_local_frame(16, |env| query_insets(env, &activity));
    match res {
        Ok(v) => v,
        Err(e) => {
            static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) { println!("JNI okraje: chyba {:?}", e); } // jen jednou (cteme 2x za sekundu)
            let _ = env.exception_clear();
            None
        }
    }
}

#[cfg(target_os = "android")]
fn query_insets(env: &mut jni::JNIEnv, activity: &jni::objects::JObject) -> jni::errors::Result<Option<(i32, i32, i32, i32)>> {
    let window = env.call_method(activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    let decor = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
    let insets = env.call_method(&decor, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[])?.l()?;
    if insets.as_raw().is_null() { return Ok(None); } // okno jeste nema okraje
    let l = env.call_method(&insets, "getSystemWindowInsetLeft", "()I", &[])?.i()?;
    let t = env.call_method(&insets, "getSystemWindowInsetTop", "()I", &[])?.i()?;
    let r = env.call_method(&insets, "getSystemWindowInsetRight", "()I", &[])?.i()?;
    let b = env.call_method(&insets, "getSystemWindowInsetBottom", "()I", &[])?.i()?;
    Ok(Some((l, t, r, b)))
}

// mimo Android zadne okraje
#[cfg(not(target_os = "android"))]
pub fn android_insets_px() -> Option<(i32, i32, i32, i32)> { None }