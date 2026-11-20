use leptos::{prelude::*};

use log::{error}; 
use core::time;

use std::io::{Read as read_io};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsValue;

use crate::reaction::utils::{invoke, tauri_listen, invoke_no_args};
use crate::reaction::types::{TimerStruct, TimerContext, UserOptions};
use crate::utils::on_key_down;


#[component]
pub fn Timer(time_to_count: i64, ended: Option<RwSignal<bool>>) -> impl IntoView{
    let timer_struct = TimerStruct::new(time_to_count);
    
    let play_sound = move || {
        spawn_local(async move {
            #[derive(Serialize, Deserialize)]
            struct SoundArgs {
                filePath: String,
            }

            let args = to_value(&SoundArgs 
                { 
                    filePath:"prepare/prepare_timer.mp3".to_string() 
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
            <div class="timer-view-card">
                {move ||  
                    {   
                        if timer_struct.time_left.get() == 0 { 
                            try_handle.get().unwrap().clear(); 
                            if let Some(end) = ended{
                                *end.write() = true;
                            }
                        };
                        view!{<div class="timer-countdown">{timer_struct.minutes.get()} : {timer_struct.seconds.get()}</div>}
                    } 
                }
            </div>
        }.into_any()
        
}

#[component]
pub fn ButtonTimer(time_to_count: i64, ended: Option<RwSignal<bool>>) -> impl IntoView{
    let timer_context = 
        use_context::<RwSignal<TimerContext>>()
        .expect("To find the count signal in context");
    
    let timer_struct = TimerStruct::new(time_to_count);
    
    // backend handlers
    let play_round_timer = move || {
        spawn_local(async move {
            #[derive(Serialize, Deserialize)]
            struct RoundSoundArgs {
                options: UserOptions,
            }

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
            #[derive(Serialize, Deserialize)]
            struct SoundArgs {
                filePath: String,
            }
            let args = to_value(&SoundArgs 
                { 
                    filePath:"prepare/end_of_round.mp3".to_string() 
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
            abort_round();
            play_end_sound();
        }
        if *timer_struct.time_left.read() > 0 {
            *timer_struct.time_left.write() = timer_struct.time_left.get() - 1;
            timer_struct.update_display();
        }

        if *timer_struct.time_left.read() == 0 {
            
           if (*timer_context.read()).user_options.round_count > 1{
                
                timer_context.update(|opt| {
                    opt.user_options.prepare_time = opt.user_options.rest_time;
                    opt.user_options.round_count -= 1;
                });
                *(ended.unwrap()).write() = false;
           }else{
                (move|| (*timer_context.write()).view_timer = false)();
           }
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

    let draw_button = RwSignal::new(false);
    Effect::new(move |_| {
        spawn_local(async move {
            
            let callback = 
                Closure::wrap(Box::new(move |_event_obj: JsValue| {
                draw_button.set(true);
            }) as Box<dyn FnMut(JsValue)>);

            tauri_listen("backend-signal", callback.as_ref().unchecked_ref()).await;
            callback.forget();
        });
    });

    let buttons_visible = RwSignal::new(false);

    let go_button = move || {
        resume_round();
        draw_button.set(false)
    };
    on_key_down(go_button);


    let pause = move || {
        try_handle.get().unwrap().clear();
        pause_round();
        *buttons_visible.write() = true
    };

    on_key_down(pause);

    view!{
        <div class="timer-page-layout">
            <div class="timer-view-card">

                <span class="timer-status">
                    {move || {
                        if timer_struct.time_left.get() == 0 { 
                            try_handle.get().unwrap().clear(); 
                            "Раунд завершен"
                        } else {
                            "Идет раунд"
                        }
                    }}
                </span>

                <div class="timer-countdown">
                    {move || format!("{:02}:{:02}", timer_struct.minutes.get(), timer_struct.seconds.get())}
                </div>
                <Show when=move || buttons_visible.get() 
                    
                    fallback=move || view! { 
                        <div class="timer-controls-row" style="grid-template-columns: 1fr;">
                            <button class="btn-primary" on:click=move |ev| {
                                ev.prevent_default();
                                pause()
                            }>
                                "Пауза"
                            </button>
                        </div>
                    }>
                    
                    <div class="timer-controls-row">
                        <button class="btn-primary" on:click=move |_| { 
                            resume_timer(); 
                            resume_round();
                            *buttons_visible.write() = false 
                        } >
                            "Продолжить"
                        </button>
                        <button class="btn-secondary" on:click=move |_| {
                            abort_round();
                            reset_timer();
                            play_round_timer();
                            resume_timer();
                            *buttons_visible.write() = false 
                        } >
                            "Заново"
                        </button>
                    </div>
                    <button class="back-link" on:click=move |_| { 
                            (*timer_context.write()).view_timer = false;
                            try_handle.get().unwrap().clear();
                            abort_round();
                        } >
                            "Вернуться к настройкам"
                    </button>
                </Show>
            </div>
            <div class="action-wrapper">
                {move || if draw_button.get() {
                    Some(view! {
                        <button 
                            class="circle-pulse-btn"
                            on:click=move |ev| { 
                                ev.prevent_default();
                                go_button()
                            }
                        >
                            "GO"
                        </button>
                    })
                } else {
                    None
                }}
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn SoundTimer() -> impl IntoView{ 

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
                .user_options.round_time ended=Some(ended)/>
        </Show>
    }
}