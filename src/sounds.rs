use leptos::{prelude::*};
use serde::{Deserialize, Serialize};

use crate::timer::SoundTimer;
use leptos_icons::{Icon};

use std::sync::LazyLock;

use icondata::Icon; 
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
    let style = "width: 3em; height: 3em; color: #374151;";
    vec![
        SoundButton { icon: &icondata::BsArrowUpLeft, style, path: "../public/directions/north_west.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowUpRight, style, path: "../public/directions/north_east.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowDownRight, style, path: "../public/directions/south_east.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowDownLeft, style, path: "../public/directions/south_west.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowUp, style, path: "../public/directions/north.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowLeft, style, path: "../public/directions/west.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowRight, style, path: "../public/directions/east.mp3".to_string() },
        SoundButton { icon: &icondata::BsArrowDown, style, path: "../public/directions/south.mp3".to_string() },
    ]
});

static COLORS_SOUNDS: LazyLock<Vec<SoundButton>> = LazyLock::new(|| {
    vec![
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #df1010;", path: "../public/colors/red.mp3".to_string() },
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #104fe2;", path: "../public/colors/blue.mp3".to_string() },
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #0e0b0b;", path: "../public/colors/black.mp3".to_string() },
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #0e6906;", path: "../public/colors/green.mp3".to_string() },
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #d6e62f;", path: "../public/colors/yellow.mp3".to_string() },
        SoundButton { icon: &icondata::BsSquareFill, style: "width: 3em; height: 3em; color: #827777;", path: "../public/colors/grey.mp3".to_string() },
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
        <p class="task-banner">"Задача: вы выполняете действие в течение заданного времени и 
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
                btn.style, 
                ArcRwSignal::new(false)))
            .collect::<Vec<_>>();
    
    let (choices, 
        set_choices) = signal(initial_choices);

    let choice_vector = move || {
        let init = if category.get() == 0{
            COLORS_SOUNDS.clone()
        }else{
            DIRECTIONS_SOUNDS.clone()
        };

        set_choices.update(move |vec| *vec = 
            init.iter()
            .map(|btn| (btn.path.clone(), 
                btn.icon.clone(),
                btn.style, 
                ArcRwSignal::new(false)))
            .collect::<Vec<_>>());

        view!{
            <div class="choice-pill-container">
                <For
                    each=move || choices.get()
                    key=|choices| choices.clone()
                    children=move |(id, icon, style, pushed)| {
                        let pushed = RwSignal::from(pushed);
                        view! {
                            <div>
                                <button id=id
                                    class="choice-pill"
                                    class:chosen=move || pushed.get()
                                    on:click=move |_| *pushed.write() = !pushed.get()
                                >
                                    <Icon icon=icon style=format!("width: 3em; height: 3em; color: {}", style) />
                                </button>
                            </div>
                        }
                    }
                />
            </div>
        }
    };


    view!{
        <div class="settings-grid">
            <div class="panel-card">
                <p class="section-title">Задайте звуковые параметры</p>
                <div class="form-group">
                    <label>"Выберите категорию сигнала:"</label>
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
                <div class="form-group">
                    <label>"Выбор звукового стимула:"</label>
                    <div class="color-pill-container">
                        {move || choice_vector}
                    </div>
                </div>
            </div>
            <div class="panel-card"> 
                <p class="section-title">Задайте временные параметры</p>
                <div class="input-row-three">
                    <div class="form-group">
                        <label>"Время на подготовку"</label>
                        <input 
                            type="number"

                            prop:value=move || (*timer_context.read()).user_options.prepare_time
                            
                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.prepare_time = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label>"Количество раундов"</label>
                        <input 
                            type="number"
                            
                            prop:value=move || (*timer_context.read()).user_options.round_count

                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.round_count = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                            
                        />
                    </div>
                    <div class="form-group">
                        <label>"Длительность раунда"</label>
                        <input 
                            type="number"

                            prop:value=move || (*timer_context.read()).user_options.round_time
                            
                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.round_time = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                </div>
                <div class="input-row-two">
                    <div class="form-group">
                        <label>"Отдых между раундами"</label>
                        <input 
                            type="number"

                            prop:value=move || (*timer_context.read()).user_options.rest_time
                            
                            on:input:target=move |ev| {
                                (*timer_context.write()).user_options.rest_time = 
                                ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    
                    <div class="form-group">
                        <label>"Пауза между сигналами"</label>
                            <input type="number" step="0.1" placeholder="3.0"
                                on:input:target=move |ev| {
                                    (*timer_context.write()).user_options.signal_freq = 
                                    ev.target().value().parse::<f64>().unwrap();

                                }
                                prop:value=timer_context.get_untracked().user_options.signal_freq
                            />
                    </div>
                </div>
                <div class="input-row-two">               
                    <div class="form-group">
                        <label>"C касанием:"</label>
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
                    </div>
                    {move || if(*timer_context.read()).user_options.on_touch == 0 {
                        view!{
                            <div class="form-group">
                                <label>"Вариативность сигнала (да/нет)"</label>
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
                            </div>
                        }.into_any()
                    }else{
                        view!{<div></div>}.into_any()
                    }
                }
                </div>
            </div>
        </div>
        <div class="footer-actions">
            <button class="btn-primary" on:click=move |_| {
                (*timer_context.write()).user_options.sounds = choices
                    .read()
                    .iter()
                    .filter(|(_, _, _, signal)|{signal.read() == true})
                    .map(|(path, _, _, _)| path)
                    .cloned()
                    .collect();
                (*timer_context.write()).view_timer = true;             
            }>
                <p>"Начать"</p>
            </button>
        </div>
        
    }
}

