use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

// Store the stop flag in Tauri State
struct WorkerState {
    is_running: Arc<AtomicBool>,
}

#[tauri::command]
fn start_task(state: State<'_, WorkerState>) {
    let running = state.is_running.clone();
    running.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            // Do your background work here
            std::thread::sleep(std::time::Duration::from_millis(500));
            println!("Working...");
        }
        println!("Thread stopped gracefully.");
    });
}

#[tauri::command]
fn stop_task(state: State<'_, WorkerState>) {
    state.is_running.store(false, Ordering::SeqCst);
}



use std::sync::{Arc, Mutex, Condvar};
use tauri::State;

// 1. Structure to control the pause/resume lifecycle
struct TaskControl {
    // Mutex holds the boolean: true = active, false = paused
    is_active: Mutex<bool>,
    // Condvar allows the thread to wait efficiently without using CPU
    condvar: Condvar,
}

struct AppState {
    control: Arc<TaskControl>,
}

#[tauri::command]
fn start_task(state: State<'_, AppState>) {
    let control = state.control.clone();
    
    // Set active state to true right away
    {
        let mut active = control.is_active.lock().unwrap();
        *active = true;
    }

    std::thread::spawn(move || {
        println!("Background thread started.");
        
        loop {
            // --- PAUSE CHECK BLOCK ---
            {
                let mut active = control.is_active.lock().unwrap();
                // If false (paused), wait until condvar is notified
                while !*active {
                    println!("Thread is paused. Sleeping...");
                    active = control.condvar.wait(active).unwrap();
                    println!("Thread woke up!");
                }
            }
            // -------------------------

            // Your actual background logic goes here
            println!("Task is running and processing data...");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

#[tauri::command]
fn pause_task(state: State<'_, AppState>) {
    println!("Pause requested.");
    let mut active = state.control.is_active.lock().unwrap();
    *active = false; // Thread will hit the while loop and sleep on next loop iteration
}

#[tauri::command]
fn resume_task(state: State<'_, AppState>) {
    println!("Resume requested.");
    let mut active = state.control.is_active.lock().unwrap();
    *active = true;
    state.control.condvar.notify_one(); // Wake up the thread
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            control: Arc::new(TaskControl {
                is_active: Mutex::new(false),
                condvar: Condvar::new(),
            }),
        })
        .invoke_handler(tauri::generate_handler![start_task, pause_task, resume_task])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
