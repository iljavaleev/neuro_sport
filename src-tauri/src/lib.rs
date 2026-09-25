// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{fs::File, option};
use std::io::BufReader;
use rodio::{Decoder, MixerDeviceSink, source::Source};
use std::io::{self, Read};
use std::io::Cursor;
use std::time::{self, Duration};
use std::thread;
use serde::Deserialize;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use std::sync::{Mutex, Arc, Condvar};
use log::{error, info, log}; 
use tokio::task::JoinHandle;


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
    pub rest_time: Option<i64>,
    pub signal_variants: Option<i32>,
    pub signal_freq: Option<f64>,
}


struct TaskControl {
    is_running: Mutex<AtomicBool>,
    is_stopped: Mutex<AtomicBool>,
    condvar: Condvar,
    abort_handle: Mutex<Option<tokio::task::AbortHandle>>,
}

struct AppState {
    control: Arc<TaskControl>,
}

impl AppState {
    fn new() -> Self{
        Self { control: Arc::new(
            TaskControl { 
                is_running: Mutex::new(AtomicBool::new(false)), 
                is_stopped: Mutex::new(AtomicBool::new(true)), 
                condvar: Condvar::new(), 
                abort_handle: Mutex::new(None), 
            }
        ) 
    }
    }
}


#[tauri::command]
async fn start_play_sound_round(state: State<'_, AppState>, options: UserOptions) -> Result<(), String> {
    let control = state.control.clone();
    
    {
        control.is_running.lock().unwrap().store(true, Ordering::Relaxed);
        control.is_stopped.lock().unwrap().store(false, Ordering::Relaxed);
    }

    let handle: JoinHandle<Result<(), String>> = tokio::spawn(async move {
        
        let hndl = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = rodio::Player::connect_new(&hndl.mixer());
        
        // если указана частота 
        if let Some(freq) = options.signal_freq{
            let n = (options.round_time as f64 / freq) as i32;
            let l = options.sounds.len();
            if l == 0{
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

            let mut ts: Vec<f64> = if Some(1) == options.signal_variants{
            
                // vec of timestamps
                let range = freq as f64 * 0.2;
                (0..n).map(|_| freq  as f64 + rand::random_range(-range..range)).collect()
            }
            else {
                (0..n).map(|_| freq  as f64).collect()
            };
            
            ts[0] = 0.5; 
            let indexes = (0..n).map(|_| {rand::random_range(0..=l-1)}); 
            let mut ind_count = 0;

            for i in indexes{
                let delay = time::Duration::from_secs_f64(ts[ind_count]);
                ind_count += 1;
                tokio::time::sleep(delay).await;
                
                {
                    
                    let mut is_stopped = control.is_stopped.lock().unwrap();
                    while is_stopped.load(Ordering::Relaxed) == true {
                        info!("stopped");
                        is_stopped = control.condvar.wait(is_stopped).unwrap();
                    }
                }
                
                let bytes_clone = sources[i].clone();
                let cursor = Cursor::new(bytes_clone);
                let source = Decoder::new(cursor).unwrap();
                
                player.append(source);
            }
        }
        
            player.sleep_until_end();
            Ok(())
        });
    
    
    *state.control.abort_handle.lock().unwrap() = Some(handle.abort_handle());
    if let Err(err_string) = handle.await{
        return Err(err_string.to_string());
    }    
    Ok(())
}


#[tauri::command]
async fn abort_play_sound_round(state: State<'_, AppState>) -> Result<(), String>{
    let Some(handle) = state
        .control
        .abort_handle
        .lock()
        .unwrap()
        .take() else{
        return Err("Handle is not ready".to_string());
    };
    handle.abort();
    Ok(())
}


#[tauri::command]
async fn pause_play_sound_round(state: State<'_, AppState>) -> Result<(), String> {
    let is_stopped = state.control.is_stopped.lock().unwrap();
    info!("stop");
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
