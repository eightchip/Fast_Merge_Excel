use std::sync::{Arc, Mutex};
use std::thread;
use crate::app::AppState;

/// AppStateはAppのstep, is_processing, 必要な一時データのみを持つ構造体として定義する（app.rsで定義/利用）
pub fn async_step_transition<F, R>(app_state: Arc<Mutex<AppState>>, next_step: u8, heavy_task: F)
where
    F: FnOnce() -> R + Send + 'static,
    R: FnOnce(&mut AppState) + Send + 'static,
{
    println!("ENTERED ASYNC STEP TRANSITION");
    println!("LOCKING MUTEX FOR is_processing");
    let mut state = app_state.lock().expect("AppState mutex poisoned (is_processing)");
    println!("LOCKED MUTEX FOR is_processing");
    state.is_processing = true;
    let app_state_clone = app_state.clone();
    println!("SPAWNING THREAD");
    thread::spawn(move || {
        println!("[async_step_transition] heavy_task start");
        // ロック外で重い処理
        let result = heavy_task();
        println!("[async_step_transition] heavy_task end, writing result");
        // 結果だけロックして反映
        println!("LOCKING MUTEX FOR result write");
        let mut state = app_state_clone.lock().expect("AppState mutex poisoned (result write)");
        println!("LOCKED MUTEX FOR result write");
        result(&mut state);
        state.is_processing = false;
        state.step = next_step;
        println!("[async_step_transition] step={} done, is_processing={}", next_step, state.is_processing);
    });
} 