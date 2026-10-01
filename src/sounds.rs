use leptos::{prelude::*};
use leptos::logging::log;
use serde::{Deserialize, Serialize};

use crate::timer::SoundTimer;
use leptos_icons::{Icon, IconProps as _IconProps};

use std::sync::LazyLock;

struct IconProps(_IconProps);

impl Clone for IconProps{
    fn clone(&self) -> Self {
        let inner = _IconProps {
            icon: self.0.icon, 
            style: self.0.style, 
            width: self.0.width, 
            height: self.0.height 
        };
        IconProps(inner)
    }
}


#[derive(Clone)]
struct SoundButton{
    icon: IconProps,
    path: String,
}


impl PartialEq for SoundButton {
    fn eq(&self, other: &SoundButton) -> bool { 
        self.icon.0.icon == other.icon.0.icon && self.path == other.path 
    }
}

impl Eq for SoundButton {}

#[derive(Debug, Clone, Deserialize, Serialize)] 
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
    fn new()-> Self {
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



static DIRECTIONS_SOUNDS: LazyLock<Vec<SoundButton>> = LazyLock::new(|| {
    let style="width: 24px; height: 24px; color: #374151;";
    vec![
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowUpLeft.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/north_west.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowUpRight.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/north_east.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowDownRight.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/south_east.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowDownLeft.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/south_west.mp3".to_string(),
        },

        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowUp.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/north.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowLeft .into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/west.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowRight.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/east.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsArrowDown.into(), style: style.to_string().into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/directions/south.mp3".to_string(),
        },
    ]
});

static COLORS_SOUNDS: LazyLock<Vec<SoundButton>> = LazyLock::new(|| {
    let create_style = |color: &str|{
        format!("width: 24px; height: 24px; color: {};", color)
    };

    vec![
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#df1010").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/red.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#104fe2").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/blue.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#0e0b0b").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/black.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#0e6906").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/green.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#d6e62f").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/yellow.mp3".to_string(),
        },
        SoundButton{
            icon: IconProps(_IconProps{icon: icondata::BsSquareFill.into(), style: create_style("#827777").into(), height:MaybeProp::default(), width:MaybeProp::default()}), 
            path: "../public/colors/grey.mp3".to_string(),
        },
    ]
});




#[component]
pub fn sound_timer_component() -> impl IntoView{
    let timer_context = TimerContext{
        user_options: UserOptions::new(),
        view_timer: false,
    };
    
    let timer_context_signal = RwSignal::new(timer_context);
    provide_context(timer_context_signal);

    view! {
        <p class="title-for-exs">"Задача: вы выполняете действие в течение заданного времени и 
        реагируете на звуковые сигналы, выполняя движения"</p>
        <Show
            when=move || {  (*timer_context_signal.read()).view_timer == true }
            fallback=|| view! { <Sounds/> }
        >
            <SoundTimer/>
        </Show>
    }
}


#[component]
pub fn sounds() -> impl IntoView{
    let timer_context = use_context::<RwSignal<TimerContext>>().expect("To find the count signal in context");

    let (category, set_category) = signal(0);
    
    let initial_choices = COLORS_SOUNDS
            .iter()
            .map(|btn| (btn.path.clone(), 
                btn.icon.clone(), 
                ArcRwSignal::new(false)))
            .collect::<Vec<_>>();
    
    let (choices, 
        set_choices) = signal(initial_choices);

    let choice_vector = move || {
        let init = if category.get() == 0{
            &COLORS_SOUNDS
        }else{
            &DIRECTIONS_SOUNDS
        };

        set_choices.update(move |vec| *vec = 
            init.iter()
            .map(|btn| (btn.path.clone(), 
                btn.icon.clone(), 
                ArcRwSignal::new(false)))
            .collect::<Vec<_>>());

        view!{
            <div class="sound-container">
                <For
                    each=move || choices.get()
                    key=|choices| choices.0.clone()
                    children=move |(id, icon, pushed)| {
                        let pushed = RwSignal::from(pushed);
                        view! {
                            <div  style="all: unset; cursor: pointer; display: inline-flex; items-center: center;">
                                <button id=id 
                                   
                                    class="choice-button"
                                    class:chosen-button=move || pushed.get()
                                    on:click=move |_| *pushed.write() = !pushed.get()
                                >
                                    {Icon(icon.0)}
                                </button>
                            </div>
                        }
                    }
                />
            </div>
        }
    };


    view!{
        <div class="sound-options">
            <div class="sound-params-container">
                <p class="section-title">Задайте звуковые параметры</p>
                <div>
                    <p class="option-title">"Выберите категорию сигнала:"</p>
                    <select
                        class="select"
                        on:change:target=move |ev| {
                            set_category.set(ev.target().value().parse().unwrap());
                        }
                        prop:value=move || category.get()
                    >
                        <option value=0>"Цвета"</option>
                        <option value=1>"Направления"</option>
                    </select>
                </div>
                <div>
                    <p class="option-title">"Выбор звукового стимула:"</p>
                    {choice_vector}
                </div>
            </div>
            <div class="time-params-container"> 
                <p class="section-title">Задайте временные параметры</p>
                <div class="time-params">
                    <div>
                        <p class="option-title">"Время на подготовку"</p>
                        <input 
                            type="text"

                            prop:value=move || (*timer_context.read()).user_options.prepare_time
                            
                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.prepare_time = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    <div>
                        <p class="option-title">"Количество раундов"</p>
                        <input 
                            type="text"
                            
                            prop:value=move || (*timer_context.read()).user_options.round_count

                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.round_count = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                            
                        />
                    </div>
                
                    <div>
                        <p class="option-title">"Длительность раунда"</p>
                        <input 
                            type="text"

                            prop:value=move || (*timer_context.read()).user_options.round_time
                            
                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.round_time = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                
                    <div>
                    <p class="option-title">"Отдых между раундами"</p>
                    <input 
                        type="text"

                        prop:value=move || (*timer_context.read()).user_options.rest_time
                        
                        on:input:target=move |ev| {
                            (*timer_context.write()).user_options.rest_time = 
                            ev.target().value().parse::<i64>().unwrap_or(0);
                        }
                    />
                    </div>
                    
                    <div>
                        <p class="option-title">"Пауза между сигналами"</p>
                            <input type="number" step="0.1" placeholder="3.0"
                                on:input:target=move |ev| {
                                    (*timer_context.write()).user_options.signal_freq = 
                                    ev.target().value().parse::<f64>().unwrap();

                                }
                                prop:value=timer_context.get_untracked().user_options.signal_freq
                            />
                    </div>
                                
                    <div>
                        <p class="option-title">"C касанием:"</p>
                        <select
                            class="select"
                            prop:value=move || (*timer_context.read()).user_options.on_touch

                            on:change:target=move |ev| {
                                (*timer_context.write()).user_options.on_touch = 
                                ev.target().value().parse().unwrap_or(0);
                            }
                        >
                            <option value=0>"Нет"</option>
                            <option value=1>"Да"</option>
                        </select>
                    
                        {move || if(*timer_context.read()).user_options.on_touch == 0 {
                            view!{<p class="option-title">"Вариативность сигнала (да/нет)"</p>
                                <select
                                    class="select"
                                    prop:value=move || (*timer_context.read()).user_options.signal_variants

                                    on:change:target=move |ev| {
                                        (*timer_context.write()).user_options.signal_variants= 
                                        Some(ev.target().value().parse().unwrap());
                                    }
                                >
                                    <option value=0>"Нет"</option>
                                    <option value=1>"Да"</option>
                                </select>
                            }.into_any()
                        }else{
                            view!{<div></div>}.into_any()
                        }
                    }
                    </div>
                </div>
            </div>
        </div>
        <div>
            <button class="start-button" on:click=move |_| {
                (*timer_context.write()).user_options.sounds = choices
                    .read()
                    .iter()
                    .filter(|(_, _, signal)|{signal.read() == true})
                    .map(|(path, _, _)| path)
                    .cloned()
                    .collect();
                (*timer_context.write()).view_timer = true;             
            }>
                <p>"Начать"</p>
            </button>
        </div>
        
    }
}

