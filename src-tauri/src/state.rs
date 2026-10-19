use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub control: Arc<TaskControl>,
}

impl AppState {
    pub fn new() -> Self{
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

pub struct TaskControl {
    pub is_running: Mutex<AtomicBool>,
    pub is_stopped: Mutex<AtomicBool>,
    pub condvar: Condvar,
    pub current_round_id: Mutex<u64>, 
}