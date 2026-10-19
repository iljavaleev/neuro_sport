// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;
pub mod state;
pub mod types;
pub mod utils;

use state::AppState;

use commands::{ 
    play_sound, 
    start_play_sound_round, 
    abort_play_sound_round, 
    pause_play_sound_round, 
    resume_play_sound_round
};

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
