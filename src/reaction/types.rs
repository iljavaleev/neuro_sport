use leptos::{prelude::*};
use crate::reaction::utils::to_minuts_and_seconds;
use serde::{Deserialize, Serialize};

use icondata::Icon; 

#[derive(Debug, Clone, Copy)]
pub struct TimerStruct{
    pub time_left: RwSignal<i64>,
    pub minutes: RwSignal<String>,
    pub seconds: RwSignal<String>, 
}

impl TimerStruct{
    pub fn new(time_to_count: i64) -> Self {
        let (minutes_display, seconds_display) = 
            to_minuts_and_seconds(time_to_count);
        
        Self{
                time_left: RwSignal::new(time_to_count),
                minutes: RwSignal::new(minutes_display),
                seconds: RwSignal::new(seconds_display),
        }
    }

    pub fn update_display(&self){
        let (minuts_display, seconds_display) = 
            to_minuts_and_seconds(self.time_left.get());
        *self.minutes.write() = minuts_display;
        *self.seconds.write() = seconds_display;
    }
 
}

#[derive(Clone)]
pub struct SoundButton {
    pub icon: &'static Icon,
    pub style: &'static str,
    pub path: String,
}

impl PartialEq for SoundButton {
    fn eq(&self, other: &SoundButton) -> bool { 
        self.icon == other.icon && self.path == other.path 
    }
}

impl Eq for SoundButton {}

#[derive(Debug, Clone, Deserialize, Serialize, Default)] 
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


impl UserOptions {
    pub fn new()-> Self {
        Self {
            sounds: Vec::<String>::new(),
            prepare_time: 5,
            round_count: 1, // если один раунд, то отдых не нужен
            round_time: 60,
            on_touch: 0,
            rest_time: 10,
            signal_variants: Some(0),
            signal_freq: 3.0,
        }
    }
}

#[derive(Clone)]
pub struct TimerContext {
    pub user_options: UserOptions,
    pub view_timer: bool,
}