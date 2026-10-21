use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::AtomicBool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;

pub struct AppState {
    pub db: sqlx::SqlitePool, 
    pub control: Arc<TaskControl>,
}

impl AppState {
     pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error>{
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        sqlx::migrate!("../migrations").run(&pool).await?;
        Ok(Self { 
            db: pool,
            control: Arc::new(TaskControl { 
                is_running: Mutex::new(AtomicBool::new(false)), 
                is_stopped: Mutex::new(AtomicBool::new(true)), 
                condvar: Condvar::new(),
                current_round_id: Mutex::new(0)
            }) 
        })
    }
}

pub struct TaskControl {
    pub is_running: Mutex<AtomicBool>,
    pub is_stopped: Mutex<AtomicBool>,
    pub condvar: Condvar,
    pub current_round_id: Mutex<u64>, 
}

