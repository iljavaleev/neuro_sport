use rodio::{Decoder};
use std::io::{Cursor};
use std::time::{self};
use std::sync::Arc;
use std::sync::atomic::{Ordering};

use log::{error, info}; 
use tauri::{AppHandle, State, Emitter};
use tauri_plugin_fs::FsExt;
use crate::state::AppState;
use crate::types::{UserOptions, ToFrontMessage, TouchOption};
use crate::utils::{get_random_delay_20, RESOURCE_DIR};

use std::convert::TryFrom;
use num_enum::TryFromPrimitive;


pub fn load_sound_from_resources(app: &AppHandle, file_path: &str) -> Result<Vec<u8>, String> {
    let base_dir = RESOURCE_DIR
        .get()
        .ok_or_else(|| "Resource directory is not initialized".to_string())?;

    let full_path = base_dir.join(file_path);

    app.fs()
        .read(full_path)
        .map_err(|e| format!("Sound load error ({file_path}): {e}"))
}


#[tauri::command]
pub async fn play_sound(app: AppHandle, file_path: String) -> Result<(), String> {
    let bytes = load_sound_from_resources(&app, &file_path)?;

    let handle = rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| {
        error!("Failed to open audio sink: {e}");
        format!("Audio device error: {e}")
    })?;

    let player = rodio::Player::connect_new(&handle.mixer());
    let cursor = Cursor::new(bytes);

    if let Ok(source) = Decoder::try_from(cursor) {
        player.append(source);
        player.sleep_until_end();
        Ok(())
    } else {
        error!("Error decoding embedded audio: {}", file_path);
        Err("Error decoding audio".to_string())
    }
}

#[tauri::command]
pub async fn start_play_sound_round(
    app: AppHandle, 
    state: State<'_, AppState>, 
    options: UserOptions
) -> Result<(), String> {
    let control = state.control.clone();
    
    let my_round_id = {
        let is_running = control.is_running.lock().unwrap();
        let is_stopped = control.is_stopped.lock().unwrap();
        let mut round_id_guard = control.current_round_id.lock().unwrap();
        
        is_running.store(true, Ordering::Relaxed);
        is_stopped.store(false, Ordering::Relaxed);
        
        *round_id_guard += 1; 
        *round_id_guard
    };

    let hndl = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|e| {
            error!("Failed to open audio sink: {e}");
            format!("Audio device error: {e}")
        })?;
    let player = rodio::Player::connect_new(&hndl.mixer());
    
    if options.sounds.is_empty() {
        let error = String::from("Empty sound list");
        error!("{}", error);
        return Err(error);
    }

    let mut sources: Vec<Arc<[u8]>> = Vec::with_capacity(options.sounds.len());
    for path in &options.sounds {
        let bytes = load_sound_from_resources(&app, &path)?;
        sources.push(Arc::from(bytes));
    }

    let get_idx = ||  {rand::random_range(0..=options.sounds.len()-1)};

    let get_time_to_wait: Box<dyn Fn() -> f64 + Send> = if Some(1) == options.signal_variants {
        Box::new(|| { get_random_delay_20(options.signal_freq) })
    } else {
        Box::new(|| { options.signal_freq as f64 })
    };

    let Ok(touch_mode) = TouchOption::try_from_primitive(options.on_touch) else {
        let error = format!("Invalid on_touch value: {}", options.on_touch);
        error!("{}", error);
        return Err(error);
    };
    
    tokio::time::sleep(time::Duration::from_secs_f64(0.5)).await;

    loop {
        
        {
            let global_id = *control.current_round_id.lock().unwrap();
            let is_running = control.is_running.lock().unwrap().load(Ordering::Relaxed);
            
            if !is_running || global_id != my_round_id {
                player.clear();
                return Ok(()); 
            }
        }
        
        if touch_mode == TouchOption::Without{
            let mut is_stopped = control.is_stopped.lock().unwrap();
            while is_stopped.load(Ordering::Relaxed) == true {
                info!("stopped");
                is_stopped = control.condvar.wait(is_stopped).unwrap();
            }
        }
        
        let bytes_slice = sources[get_idx()].clone();
        let cursor = Cursor::new(bytes_slice);
        let source = Decoder::new(cursor).map_err(|e| {
            error!("Error decoding audio: {e}");
            format!("Error decoding audio: {e}")
        })?;
        player.append(source);

        
        if touch_mode == TouchOption::With {
            app.emit("backend-signal", ToFrontMessage { message: true }).unwrap();
            let t = time::Instant::now();
            {
                let mut is_stopped = control.is_stopped.lock().unwrap();    
                is_stopped.store(true, Ordering::Relaxed); 
                
                while is_stopped.load(Ordering::Relaxed) == true {
                    is_stopped = control.condvar.wait(is_stopped).unwrap();
                }
                let delta_t = t.elapsed();
                info!("Time for push: {}", delta_t.as_secs_f64());
            }
        }
        
        let total_delay_secs = get_time_to_wait();
        let total_duration = time::Duration::from_secs_f64(total_delay_secs);

        let mut accumulated_time = time::Duration::ZERO;
        let mut last_tick = std::time::Instant::now();

        let mut ticker = tokio::time::interval(time::Duration::from_millis(50));
        // delay loop
        loop {
            ticker.tick().await;
            {
                let global_id = *control.current_round_id.lock().unwrap();
                let is_running = control.is_running.lock().unwrap().load(Ordering::Relaxed);

                if !is_running || global_id != my_round_id {
                    player.clear();
                    return Ok(());
                }
            }

            {
                let mut is_stopped = control.is_stopped.lock().unwrap();
                if is_stopped.load(Ordering::Relaxed) == true {
                    info!("Timer paused. Inner loop.");
                    
                    while is_stopped.load(Ordering::Relaxed) == true {
                        is_stopped = control.condvar.wait(is_stopped).unwrap();
                    }
                    
                    last_tick = std::time::Instant::now();
                    continue;
                }
            }
            
            let now = std::time::Instant::now();
            accumulated_time += now.duration_since(last_tick);
            last_tick = now;

            if accumulated_time >= total_duration {
                break;
            }

        }
        
    }

}


#[tauri::command]
pub async fn abort_play_sound_round(state: State<'_, AppState>) -> Result<(), String>{
    state.control.is_running.lock().unwrap().store(false, Ordering::Relaxed);
    Ok(())
}


#[tauri::command]
pub async fn pause_play_sound_round(state: State<'_, AppState>) -> Result<(), String> {
    let is_stopped = state.control.is_stopped.lock().unwrap();
    (*is_stopped).store(true, Ordering::Relaxed);
    Ok(())
}


#[tauri::command]
pub async fn resume_play_sound_round(state: State<'_, AppState>) -> Result<(), String> {
   {
        let is_stopped = state.control.is_stopped.lock().unwrap();
        (*is_stopped).store(false, Ordering::Relaxed);
    }
   state.control.condvar.notify_all();
   Ok(())
}
