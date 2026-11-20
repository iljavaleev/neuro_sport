use std::sync::OnceLock;
use std::path::PathBuf;
use tauri::{AppHandle,path::BaseDirectory, Manager};

pub static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init_resource_dir(app: &AppHandle) {
    if let Ok(dir) = app.path().resolve("", BaseDirectory::Resource) {
        let _ = RESOURCE_DIR.set(dir);
    }
}

pub fn get_random_delay_20(delay: f64) -> f64{
    let range = delay * 0.2;
    delay + rand::random_range(-range..range)
}