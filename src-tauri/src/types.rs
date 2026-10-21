use serde::{Serialize, Deserialize};
use num_enum::TryFromPrimitive;

use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;


#[derive(Serialize, Clone)]
pub struct ToFrontMessage {
    pub message: bool,
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


#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(i64)]
pub enum TouchOption {
    Without = 0,
    With = 1,
}

#[derive(Debug, Serialize, Deserialize,)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub date_of_birth: NaiveDate,
    pub sex: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize,)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i128,
    pub exercise_id: i128,
    pub date_of_birth: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize,)]
pub struct Exercise {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgilityMetrics {
    pub missed_signals: i64,
    pub avg_reaction_time_ms: f64,
    pub reaction_history: Vec<f64>, // Массив времени каждого касания
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct ExerciseStat {
    pub id: i64,
    pub session_id: i64,
    pub duration_seconds: f64,
    pub completed_rounds: i64,
    pub additional_metrics: Option<sqlx::types::Json<serde_json::Value>>, 
}