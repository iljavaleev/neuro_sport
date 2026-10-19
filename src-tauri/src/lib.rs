// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{fs::File, option};
use rodio::{Decoder, MixerDeviceSink, source::Source};
use std::io::{self, Read};
use std::io::Cursor;
use std::time::{self, Duration};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};


use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use std::sync::{Mutex, Arc, Condvar};
use log::{error, info}; 

use std::convert::TryFrom;
use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(i64)]
enum TouchOption {
    Without = 0,
    With = 1,
}


#[tauri::command]
async fn play_sound(file_path: String) -> Result<(), String> {
   
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());         
        let file = File::open(file_path).unwrap();
        
        
        if let Ok(source) = Decoder::try_from(file){
            player.append(source);
            player.sleep_until_end(); 
            Ok(())
        }else {
            error!("Error decoding file");
            Err("Error decoding file".to_string())
        }
  
}


#[derive(Debug, Clone, Deserialize)] 
pub struct UserOptions{
    pub sounds: Vec<String>,
    pub round_count: i64,
    pub prepare_time: i64,
    pub round_time: i64,
    pub on_touch: i64,
    pub rest_time: i64,
    pub signal_variants: Option<i32>,
    pub signal_freq: f64,
}


struct TaskControl {
    is_running: Mutex<AtomicBool>,
    is_stopped: Mutex<AtomicBool>,
    condvar: Condvar,
    current_round_id: Mutex<u64>, 
}

struct AppState {
    control: Arc<TaskControl>,
}

#[derive(Serialize, Clone)]
struct ToFrontMessage {
    message: bool,
}

impl AppState {
    fn new() -> Self{
        Self { control: Arc::new(
            TaskControl { 
                is_running: Mutex::new(AtomicBool::new(false)), 
                is_stopped: Mutex::new(AtomicBool::new(true)), 
                condvar: Condvar::new(),
                current_round_id: Mutex::new(0)
            }
        ) 
    }
    }
}

pub fn get_random_delay_20(delay: f64) -> f64{
    let range = delay * 0.2;
    delay + rand::random_range(-range..range)
}

#[tauri::command]
async fn start_play_sound_round(
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

    let hndl = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    let player = rodio::Player::connect_new(&hndl.mixer());
    
    if options.sounds.len() == 0{
        let error= String::from("Empty sound list");
        error!("{}", error.clone());
        return Err(error);
    }

    let sources: Vec<Arc<[u8]>> = options.sounds.iter().map(|path| {
            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let shared_bytes: Arc<[u8]> = Arc::from(buffer);
            shared_bytes
    }).collect();
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
        
        let bytes_clone = sources[get_idx()].clone();
        let cursor = Cursor::new(bytes_clone);
        let source = Decoder::new(cursor).unwrap();
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
async fn abort_play_sound_round(state: State<'_, AppState>) -> Result<(), String>{
    state.control.is_running.lock().unwrap().store(false, Ordering::Relaxed);
    Ok(())
}


#[tauri::command]
async fn pause_play_sound_round(state: State<'_, AppState>) -> Result<(), String> {
    let is_stopped = state.control.is_stopped.lock().unwrap();
    (*is_stopped).store(true, Ordering::Relaxed);
    Ok(())
}


#[tauri::command]
async fn resume_play_sound_round(state: State<'_, AppState>) -> Result<(), String> {
   {
        let is_stopped = state.control.is_stopped.lock().unwrap();
        (*is_stopped).store(false, Ordering::Relaxed);
    }
   state.control.condvar.notify_all();
   Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            play_sound, 
            start_play_sound_round, 
            abort_play_sound_round, 
            pause_play_sound_round,
            resume_play_sound_round])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
