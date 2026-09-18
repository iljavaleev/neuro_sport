// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, MixerDeviceSink, source::Source};

#[tauri::command]
fn play_native_sound(file_path: String) -> Result<(), String> {
    // Run this in a background thread so it doesn't freeze your UI
    std::thread::spawn(move || {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());
        println!("{file_path}");           
        let file = File::open(file_path).unwrap();
        // Decode that sound file into a source
        let source = Decoder::try_from(file).unwrap();
        player.append(source);
        player.sleep_until_end(); // Plays until audio is complete
    });

    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![play_native_sound])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
