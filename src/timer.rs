use leptos::{prelude::*};
use wasm_bindgen::JsValue;
use log::{info, error}; 
use core::time;

use std::io::{Read as read_io};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::sounds::{TimerContext, UserOptions};


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke", catch)]
    async fn invoke_no_args(cmd: &str) -> Result<JsValue, JsValue>;
}


#[derive(Serialize, Deserialize)]
struct SoundArgs {
    filePath: String,
}

#[derive(Serialize, Deserialize)]
struct RoundSoundArgs {
    options: UserOptions,
}


fn to_minuts_and_seconds(time_left: i64) -> (String, String){
    let minuts = time_left / 60;
    let seconds = time_left % 60;

    let minuts_display = if minuts < 10 { format!("0{minuts}")} else {
        format!("{minuts}")
    };
    let seconds_display = if seconds < 10 { format!("0{seconds}")} else {
        format!("{seconds}")
    };
    
    (minuts_display, seconds_display)
}

#[derive(Debug, Clone, Copy)]
struct TimerStruct{
    time_left: RwSignal<i64>,
    minutes: RwSignal<String>,
    seconds: RwSignal<String>, 
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

    fn update_display(&self){
        let (minuts_display, seconds_display) = 
            to_minuts_and_seconds(self.time_left.get());
        *self.minutes.write() = minuts_display;
        *self.seconds.write() = seconds_display;
    }
 
}


#[component]
pub fn timer(time_to_count: i64, ended: Option<RwSignal<bool>>) -> impl IntoView{
    let timer_struct = TimerStruct::new(time_to_count);
    
    let play_sound = move || {
        spawn_local(async move {
            let args = to_value(&SoundArgs 
                { 
                    filePath:"../public/prepare/prepare_timer.mp3".to_string() 
                }).unwrap();
            match invoke("play_sound", args).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            }
        });
    };

    
    let interval_cb = move ||{
        if *timer_struct.time_left.read() == 4 {
            play_sound();
        }
        

        if *timer_struct.time_left.read() > 0 {
            *timer_struct.time_left.write() = timer_struct.time_left.get() - 1;
            timer_struct.update_display();
        }
    };

    let try_handle =  RwSignal::new(
        set_interval_with_handle(interval_cb, time::Duration::from_secs(1))
    );

    if (try_handle.get_untracked()).is_err(){
        return view! {
            <div>"Something whent wrong"</div>
        }.into_any();
    }
    
    view!{
        {move ||  
            {   
                if timer_struct.time_left.get() == 0 { 
                    try_handle.get().unwrap().clear(); 
                    if let Some(end) = ended{
                        *end.write() = true;
                    }
                };
                view!{<div>{timer_struct.minutes.get()} : {timer_struct.seconds.get()}</div>}
            } 
        }
    }.into_any()
}


#[component]
pub fn button_timer(time_to_count: i64) -> impl IntoView{
    let timer_context = 
        use_context::<RwSignal<TimerContext>>()
        .expect("To find the count signal in context");
    
    let timer_struct = TimerStruct::new(time_to_count);
    
    // backend handlers
    let play_round_timer = move || {
        spawn_local(async move {
            let args = to_value(&RoundSoundArgs { 
                options: timer_context.get_untracked().user_options }
            ).unwrap();
            
            match invoke("start_play_sound_round", args).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            }
        });
    };

    let play_end_sound = move || {
        spawn_local(async move {
            let args = to_value(&SoundArgs 
                { 
                    filePath:"../public/prepare/end_of_round.mp3".to_string() 
                }).unwrap();

            match invoke("play_sound", args).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            }
        });
    };

    let pause_round = move || {
        info!("IN");
        spawn_local(async move {
            match invoke_no_args("pause_play_sound_round",).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            }
        });
    };

    let resume_round = move || {
        spawn_local(async move {
            match invoke_no_args("resume_play_sound_round",).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            }
            
        });
    };

    let abort_round = move || {
        spawn_local(async move {
            match invoke_no_args("abort_play_sound_round",).await{
                Ok(_) => (),
                Err(js_error) => {
                    let error_msg: String = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown Tauri backend error".to_string());
                    error!("{}", error_msg);
                }
            } 
        });
    };

    let interval_cb = move ||{
        if *timer_struct.time_left.read() == 1 {
            play_end_sound();
        }
        if *timer_struct.time_left.read() > 0 {
            *timer_struct.time_left.write() = timer_struct.time_left.get() - 1;
            timer_struct.update_display();
        }
    };

    let try_handle =  RwSignal::new(
        set_interval_with_handle(interval_cb, time::Duration::from_secs(1))
    );

    // frontend timer
    let reset_timer = move || {
       *timer_struct.time_left.write() = time_to_count;
        timer_struct.update_display();
    };
   
    let resume_timer  = move || {
        try_handle.get().unwrap().clear();
        *try_handle.write() = set_interval_with_handle(
            move || interval_cb(), //
            time::Duration::from_secs(1));
    };

    Effect::new(move |_| {
        play_round_timer();
    });

    
    view!{
        <div>
            {move ||  
                {   
                    if timer_struct.time_left.get() == 0 { 
                        try_handle.get().unwrap().clear(); 
                    };
                    view!{
                        <p>{timer_struct.minutes.get()} : {timer_struct.seconds.get()}</p>
                    }
                } 
            }
            <button on:click=move |_| {
                try_handle.get().unwrap().clear();
                pause_round()
            }>
                "Пауза"
            </button>
            <button on:click=move |_| { 
                resume_timer(); 
                resume_round() } >
                "Продолжить"
            </button>
             <button on:click=move |_| {
                abort_round();
                reset_timer();
                play_round_timer();
                resume_timer()
            } >
                "Заново"
            </button>
            <button on:click=move |_| { 
                (*timer_context.write()).view_timer = false;
                try_handle.get().unwrap().clear();
                abort_round();
            } >
                "Вернуться к настройкам"
            </button>

        </div>
    }.into_any()
}


#[component]
pub fn sound_timer() -> impl IntoView{ 
    let ended = RwSignal::new(false);
    let timer_context = 
        use_context::<RwSignal<TimerContext>>()
        .expect("To find the count signal in context");
    
    view!{
        <Show when=move || ended.get()
                fallback=move || view!{
                    <Timer 
                        time_to_count=(*timer_context.read())
                        .user_options.prepare_time ended=Some(ended) />
                }>
            <ButtonTimer 
                time_to_count=(*timer_context.read())
                .user_options.round_time />
        </Show>
    }
}