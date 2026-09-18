use leptos::{prelude::*};
use leptos::logging::log;
use wasm_bindgen::JsValue;

use core::time;

use std::io::{Read as read_io};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}


#[derive(Serialize, Deserialize)]
struct SoundArgs {
    filePath: String,
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

    fn interval_cb(&self){
        if *self.time_left.read() > 0 {
            *self.time_left.write() = self.time_left.get() - 1;
            self.update_display();
        }
    }

    fn get_handle(self) -> RwSignal<Result<IntervalHandle, JsValue>>{
        RwSignal::new(
            set_interval_with_handle(move || self.interval_cb(), time::Duration::from_secs(1))
        )
    }
    
}

 // let n = round_len / freq;
    // let l = paths.len();

    // let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    // let player = rodio::Player::connect_new(&handle.mixer());

    // // vec of shared bytes for reuse
    // let sources: Vec<Arc<[u8]>> = paths.iter().map(|path| {
    //     let mut file = File::open(path).unwrap();
    //     let mut buffer = Vec::new();
    //     file.read_to_end(&mut buffer).unwrap();
    //     let shared_bytes: Arc<[u8]> = Arc::from(buffer);
    //     shared_bytes
    // }).collect();
    
    // // vec of timestamps
    // let range = freq as f64 * 0.2;
    // let mut ts: Vec<f64> = (0..n).map(|_| freq  as f64 + rand::random_range(-range..range)).collect();
    // ts[0] = 0.0;

    // // vec of timestamp indexes
    // let indexes = (0..n).map(|_| {rand::random_range(0..=l-1)}); 
    // let mut ind_count = 0;

    // for i in indexes{
    //     let bytes_clone = sources[i].clone();
    //     let cursor = Cursor::new(bytes_clone);
    //     let source = Decoder::new(cursor).unwrap();

    //     let delay = time::Duration::from_secs_f64(ts[ind_count]);
    //     ind_count += 1;

    //     thread::sleep(delay);
    //     player.append(source);
        
        
    // }
    // player.sleep_until_end();

use web_sys::{HtmlAudioElement, MouseEvent};
use std::io::BufReader;


#[component]
pub fn timer(time_to_count: i64, ended: Option<RwSignal<bool>>) -> impl IntoView{
    let timer_struct = TimerStruct::new(time_to_count);
    
    let play_sound = move || {
        spawn_local(async move {
            let args = to_value(&SoundArgs { filePath:"../public/prepare/prepare_timer.mp3".to_string() }).unwrap();
            invoke("play_native_sound", args).await.as_f64().unwrap();
        });
    };

    // let play_sound = move || {
    //     // Path to your audio file (e.g., in your public/ assets folder)
    //     if let Ok(audio) = HtmlAudioElement::new_with_src("/prepare_timer.mp3") {
    //         let _ = audio.play();
    //     }else {
    //         log!("Not");
    //     }
    // };
    
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
    let timer_context = use_context::<RwSignal<TimerContext>>().expect("To find the count signal in context");

    
    let timer_struct = TimerStruct::new(time_to_count);

    let try_handle = timer_struct.get_handle();

    if (try_handle.get_untracked()).is_err(){
        return view! {
            <div>Something whent wrong</div>
        }.into_any();
    }


    let reset_timer = move || {
       *timer_struct.time_left.write() = time_to_count;
        timer_struct.update_display();
    };
   
    let resume_timer  = move || {
        try_handle.get().unwrap().clear();
        *try_handle.write() = set_interval_with_handle(
        move || timer_struct.interval_cb(), 
        time::Duration::from_secs(1));
    };

    
    view!{
        <div>
            {move ||  
                {   
                    if timer_struct.time_left.get() == 0 { try_handle.get().unwrap().clear(); };
                    view!{<p>{timer_struct.minutes.get()} : {timer_struct.seconds.get()}</p>}
                } 
            }
            <button on:click=move |_| try_handle.get().unwrap().clear()>
                "Пауза"
            </button>
            <button on:click=move |_| resume_timer() >
                "Продолжить"
            </button>
             <button on:click=move |_| {reset_timer(); resume_timer()} >
                "Заново"
            </button>
            <button on:click=move |_| { 
                (*timer_context.write()).view_timer = false;
                try_handle.get().unwrap().clear();
            } >
                "Вернуться к настройкам"
            </button>

        </div>
    }.into_any()
}


use crate::sounds::TimerContext;

#[component]
pub fn sound_timer() -> impl IntoView{ 
    let ended = RwSignal::new(false);
    let timer_context = use_context::<RwSignal<TimerContext>>().expect("To find the count signal in context");


    view!{
         <Show when=move || ended.get()
                fallback=move || view!{<Timer time_to_count=(*timer_context.read()).user_options.prepare_time ended=Some(ended)/>}>
            <ButtonTimer time_to_count=(*timer_context.read()).user_options.round_time/>
        </Show>
    }
}