use leptos::{prelude::*};
use leptos::logging::log;

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

#[derive(Debug, Clone)] 
pub struct UserOptions{
    pub sounds: Vec<String>,
    pub round_count: i64,
    pub prepare_time: i64,
    pub round_time: i64,
    pub rest_time: Option<i64>,
    pub signal_variants: Option<i32>,
    pub signal_freq: Option<f64>,
}

impl UserOptions {
    fn new()-> Self {
        Self {
            sounds: Vec::<String>::new(),
            prepare_time: 10,
            round_count: 1, // если один раунд, то отдых не нужен
            round_time: 60,
            rest_time: Some(10),
            signal_variants: Some(0),
            signal_freq: Some(3.0),
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
        SoundButton{ label: "Красное".to_string(), path: "path1".to_string() },
        SoundButton{ label: "Синее".to_string(), path: "path2".to_string() },
        SoundButton{ label: "Черное".to_string(), path: "path2".to_string() },
        SoundButton{ label: "Зеленое".to_string(), path: "path2".to_string() },

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
        <h1>"Описание: вы выполняете действие в течение заданного времени и реагируете на звуковые сигналы, выполняя движения"</h1>
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
    let (touch, set_touch) = signal(0);


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
            <div>
                <ul>
                    <For
                        each=move || choices.get()
                        key=|choices| choices.0.clone()
                        children=move |(id, label, pushed)| {
                            let pushed = RwSignal::from(pushed);
                            view! {
                                <li>
                                    <button id=id
                                        on:click=move |_| *pushed.write() = 
                                        !pushed.get()
                                    >
                                        {pushed}{label}
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
        }
    };


    view!{
        
        <div>
            <p>"Выберите категорию сигнала:"</p>
            <select
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
            <p>"Выбор звукового стимула:"</p>
            {choice_vector}
        </div>
        
        <div>
            <p>"Количество раундов"</p>
            <input 
                type="text"
                
                prop:value=move || (*timer_context.read()).user_options.round_count

                on:input:target=move |ev| {
                    (*timer_context.write()).user_options.round_count = 
                    ev.target().value().parse::<i64>().unwrap_or(0);
                }
                
            />
            <p>"Время на подготовку"</p>
            <input 
                type="text"

                prop:value=move || (*timer_context.read()).user_options.prepare_time
                
                on:input:target=move |ev| {
                    (*timer_context.write()).user_options.prepare_time = 
                    ev.target().value().parse::<i64>().unwrap_or(0);
                }
            />
            <p>"Длительность раунда"</p>
            <input 
                type="text"

                prop:value=move || (*timer_context.read()).user_options.round_time
                
                on:input:target=move |ev| {
                    (*timer_context.write()).user_options.round_time = 
                    ev.target().value().parse::<i64>().unwrap_or(0);
                }
            />
            <p>"Отдых между раундами"</p>
            <input 
                type="text"

                prop:value=move || (*timer_context.read()).user_options.rest_time
                
                on:input:target=move |ev| {
                    (*timer_context.write()).user_options.rest_time = 
                    Some(ev.target().value().parse::<i64>().unwrap_or(0));
                }
            />
        </div>
        <div>
            <p>"C касанием:"</p>
            <select
                prop:value=move || *touch.read()

                on:change:target=move |ev| {
                    set_touch.set(ev.target().value().parse().unwrap());
                }
            >
                <option value=0>"Нет"</option>
                <option value=1>"Да"</option>
            </select>
        </div>
        
        <div>
            {move || if touch.get() == 0 {
                view!{<p>"Вариативность сигнала (да/нет)"</p>
                    <select
                        prop:value=move || (*timer_context.read()).user_options.signal_variants

                        on:change:target=move |ev| {
                            (*timer_context.write()).user_options.signal_variants= 
                            Some(ev.target().value().parse().unwrap());
                        }
                    >
                        <option value=0>"Нет"</option>
                        <option value=1>"Да"</option>
                    </select>
                <p>"Частота сигнала"</p>
                <input type="number"
                    on:input:target=move |ev| {
                        (*timer_context.write()).user_options.signal_freq = 
                        Some(ev.target().value().parse::<f64>().unwrap());

                    }
                    prop:value=(*timer_context.read()).user_options.signal_freq
                />}.into_any()
            }else{
                view!{<div></div>}.into_any()
            }
        }
        </div>
        <div>
            <p>"Начать тренировку!"</p>
            <button on:click=move |_| {
                (*timer_context.write()).user_options.sounds = choices
                    .read()
                    .iter()
                    .filter(|(_, _, signal)|{signal.read() == true})
                    .map(|(_, path, _)| path)
                    .cloned()
                    .collect();
                (*timer_context.write()).view_timer = true;             
            }>
                <p>"Начать"</p>
            </button>
        </div>
        
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
    