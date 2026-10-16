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
        let mut is_running = control.is_running.lock().unwrap();
        let mut is_stopped = control.is_stopped.lock().unwrap();
        let mut round_id_guard = control.current_round_id.lock().unwrap();
        
        is_running.store(true, Ordering::Relaxed);
        is_stopped.store(false, Ordering::Relaxed);
        
        // Увеличиваем ID. Этот номер будет принадлежать ТОЛЬКО этому вызову
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


    if options.on_touch == 0{
                
        tokio::time::sleep(time::Duration::from_secs_f64(0.5)).await;
        loop{
            
            {
                let global_id = *control.current_round_id.lock().unwrap();
                let is_running = control.is_running.lock().unwrap().load(Ordering::Relaxed);
                
                if !is_running || global_id != my_round_id {
                    player.clear();
                    return Ok(()); // Старый поток спокойно умирает здесь
                }
            }
            
            {
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

            let delay = time::Duration::from_secs_f64(get_time_to_wait());
            tokio::time::sleep(delay).await;

            {
                let global_id = *control.current_round_id.lock().unwrap();
                if global_id != my_round_id {
                    player.clear();
                    return Ok(());
                }
            }

            
        }
    }else{
        tokio::time::sleep(time::Duration::from_secs_f64(0.5)).await;
        loop{
            
            {
                if  control.is_running.lock().unwrap().load(Ordering::Relaxed) == false{
                    player.clear();
                    return Ok(());
                }
            }

            let bytes_clone = sources[get_idx()].clone();
            let cursor = Cursor::new(bytes_clone);
            let source = Decoder::new(cursor).unwrap();
            
            player.append(source);
            app.emit("backend-signal", ToFrontMessage {
                message:true,
            }).unwrap();
            
            {
                let mut is_stopped = control.is_stopped.lock().unwrap();    
                is_stopped.store(true, Ordering::Relaxed); 
                // фронт рисует кнопку
                while is_stopped.load(Ordering::Relaxed) == true {
                    is_stopped = control.condvar.wait(is_stopped).unwrap();
                }
            }

            let delay = time::Duration::from_secs_f64(get_time_to_wait());
            tokio::time::sleep(delay).await;
            
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
        (*is_stopped).store(false, Ordering::SeqCst);
    }
   state.control.condvar.notify_one();
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
