// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{fs::File, option};
use std::io::BufReader;
use rodio::{Decoder, MixerDeviceSink, source::Source};
use std::io::{self, Read};
use std::io::Cursor;
use std::time;
use std::thread;
use serde::Deserialize;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use std::sync::{Mutex, Arc, Condvar};



#[tauri::command]
fn play_sound(file_path: String) -> Result<(), String> {
    // Run this in a background thread so it doesn't freeze your UI
    std::thread::spawn(move || {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());         
        let file = File::open(file_path).unwrap();
        // Decode that sound file into a source
        let source = Decoder::try_from(file).unwrap();
        player.append(source);
        player.sleep_until_end(); // Plays until audio is complete
    });

    Ok(())
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


#[tauri::command]
fn start_play_sound_round(options: UserOptions, state: State<'_, AppState>) -> Result<(), String> {
    let control = state.control.clone();
    
    {
        control.is_running.lock().unwrap().store(true, Ordering::Relaxed);
        control.is_stopped.lock().unwrap().store(false, Ordering::Relaxed);
    }
    let handle = tokio::spawn(async move {
        
        let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());
        
        // если указана частота 
        if let Some(freq) = options.signal_freq{
            let n = (options.round_time as f64 / freq) as i32;
            let l = options.sounds.len();
            
            let sources: Vec<Arc<[u8]>> = options.sounds.iter().map(|path| {
                let mut file = File::open(path).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                let shared_bytes: Arc<[u8]> = Arc::from(buffer);
                shared_bytes
            }).collect();

            let mut ts = Vec::<f64>::new();

            match options.signal_variants{
                // если есть вариативность исполнения
                Some(1) => {
                    // vec of timestamps
                    let range = freq as f64 * 0.2;
                    ts = (0..n).map(|_| freq  as f64 + rand::random_range(-range..range)).collect();
                },
                _ => {
                    ts = (0..n).map(|_| freq  as f64).collect();
                }
            }
            ts[0] = 0.0; 
            let indexes = (0..n).map(|_| {rand::random_range(0..=l-1)}); 
            let mut ind_count = 0;

            for i in indexes{
                {
                    let mut is_stopped = control.is_stopped.lock().unwrap();
                    while is_stopped.load(Ordering::Relaxed) == true {
                        is_stopped = control.condvar.wait(is_stopped).unwrap();
                    }
                }
                
                
                
                let bytes_clone = sources[i].clone();
                let cursor = Cursor::new(bytes_clone);
                let source = Decoder::new(cursor).unwrap();

                let delay = time::Duration::from_secs_f64(ts[ind_count]);
                ind_count += 1;

                thread::sleep(delay);
                player.append(source);
            }
        }
        
        
        player.sleep_until_end();

        });
    
    
    *state.control.abort_handle.lock().unwrap() = Some(handle.abort_handle());    
    Ok(())
}


#[tauri::command]
fn abort_play_sound_round(state: State<'_, AppState>) {
    if let Some(handle) = state.control.abort_handle.lock().unwrap().take() {
        handle.abort();
    }
}


#[tauri::command]
fn pause_play_sound_round(state: State<'_, AppState>) {
    let is_stopped = state.control.is_stopped.lock().unwrap();
    (*is_stopped).store(true, Ordering::Relaxed);
}


#[tauri::command]
fn resume_play_sound_round(state: State<'_, AppState>) {
   {
        let is_stopped = state.control.is_stopped.lock().unwrap();
        (*is_stopped).store(false, Ordering::Relaxed);
    }
   state.control.condvar.notify_one();
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            play_sound, 
            start_play_sound_round, 
            abort_play_sound_round, 
            pause_play_sound_round,
            resume_play_sound_round])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
