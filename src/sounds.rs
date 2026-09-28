use leptos::{prelude::*};
use leptos::logging::log;
use serde::{Deserialize, Serialize};

use crate::timer::SoundTimer;

use std::sync::LazyLock;

#[derive(Debug, Clone, Hash)]
struct SoundButton{
    label: String,
    path: String,
}

impl PartialEq for SoundButton {
    fn eq(&self, other: &SoundButton) -> bool { 
        self.label == other.label && self.path == other.path 
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
    vec![
        SoundButton{ label: "Влево".to_string(), path: "path1".to_string() },
        SoundButton{ label: "Вправо".to_string(), path: "path2".to_string() },
        SoundButton{ label: "Вверх".to_string(), path: "path2".to_string() },
        SoundButton{ label: "Вниз".to_string(), path: "path2".to_string() },
    ]
});

static COLORS_SOUNDS: LazyLock<Vec<SoundButton>> = LazyLock::new(|| {
    vec![
        SoundButton{ label: "Красный".to_string(), path: "../public/colors/red.mp3".to_string() },
        SoundButton{ label: "Синий".to_string(), path: "../public/colors/blue.mp3".to_string() },
        SoundButton{ label: "Черный".to_string(), path: "../public/colors/black.mp3".to_string() },
        SoundButton{ label: "Зеленый".to_string(), path: "../public/colors/green.mp3".to_string() },
        SoundButton{ label: "Желтый".to_string(), path: "../public/colors/yellow.mp3".to_string() },
        SoundButton{ label: "Серый".to_string(), path: "../public/colors/grey.mp3".to_string() }
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
                btn.label.clone(), 
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
                btn.label.clone(), 
                ArcRwSignal::new(false)))
            .collect::<Vec<_>>());

        view!{
            <div class="sound-container">
                
                <For
                    each=move || choices.get()
                    key=|choices| choices.0.clone()
                    children=move |(id, label, pushed)| {
                        let pushed = RwSignal::from(pushed);
                        view! {
                            <div>
                                <button id=id class="btn"
                                    on:click=move |_| *pushed.write() = 
                                    !pushed.get()
                                >
                                    {pushed}{label}
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

