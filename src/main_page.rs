use leptos::ev::keydown;
use leptos::{prelude::*};
use std::collections::HashMap;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use crate::utils::on_key_down;


#[derive(Clone)]
pub struct ProgramChoice {
    pub description: &'static str,
    pub link: &'static str,
}

#[component]
pub fn MainPage() -> impl IntoView {
    let exs_type = RwSignal::new(0);
    let selected_link = RwSignal::new("/remember".to_string());
    let navigate = use_navigate();

    let all_choices = StoredValue::new(HashMap::from([
        (
            0,
            vec![ProgramChoice {
                description: "Запомни",
                link: "/remember",
            }],
        ),
        (
            1,
            vec![ProgramChoice {
                description: "Будь внимателен",
                link: "/attention",
            }],
        ),
        (
            2,
            vec![ProgramChoice {
                description: "Исполнение",
                link: "/make-it",
            }],
        ),
        (
            3,
            vec![ProgramChoice {
                description: "Звуковой таймер",
                link: "/sound-timer",
            }],
        ),
    ]));

    Effect::new(move |_| {
        let current_type = exs_type.get();
        all_choices.with_value(|map| {
            if let Some(first) = map.get(&current_type).and_then(|list| list.first()) {
                selected_link.set(first.link.to_string());
            }
        });
    });

    let start_exercise = move || {
        let link = selected_link.get();
        if !link.is_empty() {
            navigate(&link, NavigateOptions::default());
        }
    };

    on_key_down(start_exercise.clone());
    
    view! {
        <main class="main-content auth-page">
            <div class="auth-top-wrapper">
                <div class="panel-card auth-card" style="width: 100%; max-width: 27.5rem;">
                    <h2>"Выбор тренировки"</h2>
                    
                    <div class="form-group" style="margin-bottom: 1.5rem;">
                        <label class="form-label">"Выберите тип упражнения"</label>
                        <select
                            class="select"
                            prop:value=move || exs_type.get().to_string()
                            on:change=move |ev| {
                                if let Ok(new_val) = event_target_value(&ev).parse::<i32>() {
                                    exs_type.set(new_val);
                                }
                            }
                        >
                            <option value=0>"Память"</option>
                            <option value=1>"Внимание"</option>
                            <option value=2>"Исполнение"</option>
                            <option value=3>"Скорость обработки информации"</option>
                        </select>
                    </div>

                    <div class="form-group" style="margin-bottom: 1.5rem;">
                        <label class="form-label">"Выберите упражнение"</label>
                        <select
                            class="select"
                            prop:value=move || selected_link.get()
                            on:change=move |ev| {
                                selected_link.set(event_target_value(&ev));
                            }
                        >
                            {move || {
                                let current_type = exs_type.get();
                                all_choices.with_value(|map| {
                                    map.get(&current_type)
                                        .into_iter()
                                        .flatten()
                                        .map(|choice| {
                                            view! {
                                                <option value=choice.link>{choice.description}</option>
                                            }
                                        })
                                        .collect_view()
                                })
                            }}
                        </select>
                    </div>

                    <div class="form-footer-actions">
                        <button class="btn-primary" on:click=move |ev|{
                            ev.prevent_default();
                            start_exercise()
                        }>
                            "Перейти к упражнению"
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}