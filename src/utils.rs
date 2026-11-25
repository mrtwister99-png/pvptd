use bevy::prelude::*;
use std::collections::VecDeque;
use crate::constants::{COLS, ROWS, CELL, LANE_W, LANE_H, LANE_Y};
use crate::components::Side;

// policko na lajne dane strany (None = bod lezi mimo jeji mrizku)
pub fn world_to_grid(side: Side, x: f32, y: f32) -> Option<(usize, usize)> {
    let left = side.lane_x() - LANE_W / 2.0;
    let bottom = LANE_Y - LANE_H / 2.0;
    if x < left || x > left + LANE_W { return None; }
    if y < bottom || y > bottom + LANE_H { return None; }
    let col = ((x - left) / CELL).floor() as usize;
    let row = ((y - bottom) / CELL).floor() as usize;
    if col >= COLS || row >= ROWS { return None; }
    Some((row, col))
}

pub fn grid_to_world(side: Side, row: usize, col: usize) -> Vec2 {
    let left = side.lane_x() - LANE_W / 2.0;
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

pub fn path_to_adjacent_blocked_cell(
    blocked: &[[bool; COLS]; ROWS],
    start: (usize, usize),
    target: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    if start.0 >= ROWS
        || start.1 >= COLS
        || target.0 >= ROWS
        || target.1 >= COLS
        || blocked[start.0][start.1]
        || !blocked[target.0][target.1]
    {
        return None;
    }

    let mut parent = [[None; COLS]; ROWS];
    let mut visited = [[false; COLS]; ROWS];
    let mut queue = VecDeque::new();
    visited[start.0][start.1] = true;
    queue.push_back(start);

    let dirs = [(1isize, 0isize), (0, -1), (0, 1), (-1, 0)];
    let mut end = None;
    while let Some((row, col)) = queue.pop_front() {
        if dirs.iter().any(|&(dr, dc)| {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            nr == target.0 as isize && nc == target.1 as isize
        }) {
            end = Some((row, col));
            break;
        }

        for &(dr, dc) in &dirs {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize {
                continue;
            }
            let next = (nr as usize, nc as usize);
            if blocked[next.0][next.1] || visited[next.0][next.1] {
                continue;
            }
            visited[next.0][next.1] = true;
            parent[next.0][next.1] = Some((row, col));
            queue.push_back(next);
        }
    }

    let mut route = Vec::new();
    let mut current = end?;
    route.push(current);
    while current != start {
        current = parent[current.0][current.1]?;
        route.push(current);
    }
    route.reverse();
    Some(route)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardian_route_reaches_the_block_from_a_walkable_side() {
        let mut blocked = [[false; COLS]; ROWS];
        let start = (8, 3);
        let target = (6, 3);
        blocked[target.0][target.1] = true;
        blocked[7][3] = true;

        let route = path_to_adjacent_blocked_cell(&blocked, start, target).unwrap();

        assert_eq!(route.first(), Some(&start));
        assert_eq!(route.last(), Some(&(6, 2)));
        assert!(route.iter().all(|&(row, col)| !blocked[row][col]));
        assert_eq!(route.last().unwrap().0.abs_diff(target.0) + route.last().unwrap().1.abs_diff(target.1), 1);
    }

    #[test]
    fn guardian_route_returns_none_when_its_start_is_blocked() {
        let mut blocked = [[false; COLS]; ROWS];
        blocked[8][3] = true;

        assert!(path_to_adjacent_blocked_cell(&blocked, (8, 3), (6, 3)).is_none());
    }
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