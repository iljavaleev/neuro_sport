use leptos::{prelude::*};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExerciseCategory {
    SoundTimer,
    Remember,
    Attention,
    MakeIt,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExerciseDescription { 
    pub title: &'static str,
    pub goals: &'static [&'static str],  
    pub steps: &'static [&'static str],
}


impl Default for ExerciseDescription{
    fn default() -> Self{
        Self{
            title: "Упражнение",
            goals: &["Описание скоро появится."],
            steps: &[],
        }
    }
}


impl ExerciseDescription {
    pub fn to_string(&self) -> String{
        let to_list = |arr: &'static [&'static str]|{
            arr.iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n")
        };
        
        let steps = to_list(self.steps);
        let goals = to_list(self.goals);
            
        format!("title: {}\ngoal: {}\ndecription: {}", 
            self.title, 
            goals, 
            steps
        )
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExerciseItem {
    pub id: u64,              
    pub category: ExerciseCategory,      
    pub description: ExerciseDescription,
    pub collect_stats: bool,
}


impl Default for ExerciseItem{
    fn default() -> Self{
        Self{
            id: 0000,
            category: ExerciseCategory::Attention,
            description: ExerciseDescription::default(),
            collect_stats: false,
        }
    }
}


pub const ALL_EXERCISES: &[ExerciseItem] = &[
    ExerciseItem {
        id: 1001,
        category: ExerciseCategory::SoundTimer,
        description: ExerciseDescription{
            title: "Делай по сигналу!",
            goals: &[
                    "Развитие динамического равновесия, сердечно-сосудистой выносливости",
                    "Развитие слуховой внимательности",
                    "Развитие избирательной внимательности", 
                ],
            steps: &[
                "Выберите звуковой стимул из предствленных категорий (цвета, направления)",
                "Для каждого стимула выберите движение, которое вы будете выполнять",
                "Услышав стимул, выполняйте соответсвующее движение до следующего сигнала",
                "Задайте необходимые временные параметры и нажмите 'Начать'",
            ],
        },
        collect_stats: true,
    },
];


pub fn get_exercises_by_category(category: ExerciseCategory) -> Vec<&'static ExerciseItem> {
    ALL_EXERCISES
        .iter()
        .filter(|ex| ex.category == category)
        .collect()
}


pub fn find_exercise_by_id(id: u64) -> Option<&'static ExerciseItem> {
    ALL_EXERCISES.iter().find(|ex| ex.id == id)
}


#[component]
pub fn ExerciseBanner(exercise_id: u64) -> impl IntoView {
    let default_exercise = ExerciseItem::default();
    let exercise = find_exercise_by_id(exercise_id).unwrap_or(&default_exercise);
    
    let desc = exercise.description;
    let (is_open, set_open) = signal(true);
    let (font_rem, set_font_rem) = signal(0.875_f32);

    view! {
        <div class="task-banner">
            <div class="task-banner-header">
                <strong class="task-banner-title">{desc.title}</strong>

                <div class="task-banner-controls">
                    <button
                        type="button"
                        class="banner-ctrl-btn"
                        title="Уменьшить шрифт"
                        on:click=move |_| set_font_rem.update(|s| *s = (*s - 0.1).max(0.75))
                    >
                        "A-"
                    </button>
                    <button
                        type="button"
                        class="banner-ctrl-btn"
                        title="Увеличить шрифт"
                        on:click=move |_| set_font_rem.update(|s| *s = (*s + 0.1).min(1.5))
                    >
                        "A+"
                    </button>
                    <button
                        type="button"
                        class="banner-ctrl-btn"
                        on:click=move |_| set_open.update(|v| *v = !*v)
                    >
                        {move || if is_open.get() { "Свернуть" } else { "Развернуть" }}
                    </button>
                </div>
            </div>

            <Show when=move || is_open.get()>
                <div
                    class="task-banner-body"
                    style=move || format!("font-size: {}rem;", font_rem.get())
                >
                    {(!desc.goals.is_empty()).then(|| view! {
                        <div class="banner-section">
                            <h4 class="banner-section-title">"Цели:"</h4>
                            <ol class="exercise-list">
                                {desc.goals.iter().map(|&goal| view! { <li>{goal}</li> }).collect_view()}
                            </ol>
                        </div>
                    })}

                    {(!desc.steps.is_empty()).then(|| view! {
                        <div class="banner-section">
                            <h4 class="banner-section-title">"Инструкция:"</h4>
                            <ol class="exercise-list">
                                {desc.steps.iter().map(|&step| view! { <li>{step}</li> }).collect_view()}
                            </ol>
                        </div>
                    })}
                </div>

            </Show>
        </div>
    }
}