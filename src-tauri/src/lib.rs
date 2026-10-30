// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;
pub mod state;
pub mod types;
pub mod utils;

use tauri::Manager;
use state::AppState;

use commands::{ 
    play_sound, 
    start_play_sound_round, 
    abort_play_sound_round, 
    pause_play_sound_round, 
    resume_play_sound_round, 
    create_user, 
    check_user_exists
};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            play_sound, 
            start_play_sound_round, 
            abort_play_sound_round, 
            pause_play_sound_round,
            resume_play_sound_round,
            create_user,
            check_user_exists

        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to resolve local app data directory");
            
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory container");

            let db_path = app_data_dir.join("agility_timer.db");

            let state = tauri::async_runtime::block_on(async {
                AppState::new(db_path)
                    .await
                    .expect("Failed to initialize database")
            });

            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
