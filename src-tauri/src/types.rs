use serde::{Serialize, Deserialize};
use num_enum::TryFromPrimitive;


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
