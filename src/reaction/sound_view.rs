use leptos::{prelude::*};
use crate::reaction::SoundTimer;
use crate::reaction::types::{SoundButton, TimerContext, UserOptions};
use leptos_icons::{Icon};
use std::sync::LazyLock;
use std::collections::HashSet;


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
pub fn SoundTimerComponent() -> impl IntoView{
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
pub fn Sounds() -> impl IntoView {
     let timer_context = use_context::<RwSignal<TimerContext>>()
        .expect("To find the count signal in context");

    let (category, set_category) = signal(0);
    
    let selected_paths = RwSignal::new(HashSet::<String>::new());

    let current_buttons = move || {
        if category.get() == 0 {
            COLORS_SOUNDS.as_slice()
        } else {
            DIRECTIONS_SOUNDS.as_slice()
        }
    };


    view! {
        <div class="settings-grid">
            <div class="panel-card">
                <p class="section-title">"Задайте звуковые параметры"</p>
                <div class="form-group">
                    <label>"Выберите категорию сигнала:"</label>
                    <select
                        class="select"
                        on:change:target=move |ev| {
                            if let Ok(val) = ev.target().value().parse::<i32>() {
                                set_category.set(val);
                                selected_paths.update(|set| set.clear());
                            }
                        }
                        prop:value=move || category.get()
                    >
                        <option value=0>"Цвета"</option>
                        <option value=1>"Направления"</option>
                    </select>
                </div>

                 <div class="form-group">
                    <label>"Выбор звукового стимула:"</label>
                    <div class="choice-pill-container">
                        <For
                            each=move || current_buttons().iter()
                            key=|btn| btn.path.as_str()
                            children=move |btn| {
                                let path_for_class = btn.path.clone();
                                let path_for_click = btn.path.clone();

                                view! {
                                    <div>
                                        <button
                                            id=btn.path.clone()
                                            type="button"
                                            class="choice-pill"
                                            class:chosen=move || {
                                                selected_paths.with(|set| set.contains(&path_for_class))
                                            }
                                            on:click=move |_| {
                                                selected_paths.update(|set| {
                                                    if !set.remove(&path_for_click) {
                                                        set.insert(path_for_click.clone());
                                                    }
                                                });
                                            }
                                        >
                                            <Icon icon=*btn.icon style=btn.style />
                                        </button>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>

            <div class="panel-card"> 
                <p class="section-title">"Задайте временные параметры"</p>
                <div class="input-row-three">
                    <div class="form-group">
                        <label>"Время на подготовку"</label>
                        <input 
                            type="number"
                            prop:value=move || timer_context.read().user_options.prepare_time
                            on:input:target=move |ev| {
                                timer_context.write().user_options.prepare_time = 
                                    ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label>"Количество раундов"</label>
                        <input 
                            type="number"
                            prop:value=move || timer_context.read().user_options.round_count
                            on:input:target=move |ev| {
                                timer_context.write().user_options.round_count = 
                                    ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label>"Длительность раунда"</label>
                        <input 
                            type="number"
                            prop:value=move || timer_context.read().user_options.round_time
                            on:input:target=move |ev| {
                                timer_context.write().user_options.round_time = 
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
                            prop:value=move || timer_context.read().user_options.rest_time
                            on:input:target=move |ev| {
                                timer_context.write().user_options.rest_time = 
                                    ev.target().value().parse::<i64>().unwrap_or(0);
                            }
                        />
                    </div>
                    
                    <div class="form-group">
                        <label>"Пауза между сигналами"</label>
                        <input 
                            type="number" 
                            step="0.1" 
                            placeholder="3.0"
                            prop:value=move || timer_context.read().user_options.signal_freq
                            on:input:target=move |ev| {
                                // Заменили .unwrap() на безопасный .unwrap_or(0.0)
                                timer_context.write().user_options.signal_freq = 
                                    ev.target().value().parse::<f64>().unwrap_or(0.0);
                            }
                        />
                    </div>
                </div>

                <div class="input-row-two">               
                    <div class="form-group checkbox-container">
                        <label class="checkbox-label">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || timer_context.read().user_options.on_touch == 1
                                on:change:target=move |ev| {
                                    let is_checked = ev.target().checked();
                                    timer_context.write().user_options.on_touch = if is_checked { 1 } else { 0 };
                                }
                            />
                            "С касанием"
                        </label>
                    </div>
                    <div class="form-group checkbox-container">
                        <label class="checkbox-label">
                            <input
                                type="checkbox"
                                class="checkbox"
                                prop:checked=move || timer_context.read().user_options.signal_variants == Some(1)
                                on:change:target=move |ev| {
                                    let is_checked = ev.target().checked();
                                    timer_context.write().user_options.signal_variants = 
                                        Some(if is_checked { 1 } else { 0 });
                                }
                            />
                            "Вариативность сигнала"
                        </label>
                    </div> 
                </div>
            </div>
        </div>
        <div class="footer-actions">
            <button 
                type="button"
                class="btn-primary" 
                on:click=move |_| {
                    let sounds: Vec<String> = selected_paths.with_untracked(|set| {
                        set.iter().cloned().collect()
                    });

                    timer_context.update(|ctx| {
                        ctx.user_options.sounds = sounds;
                        ctx.view_timer = true;
                    });
                }
            >
                "Начать"
            </button>
        </div>
     
}
}
