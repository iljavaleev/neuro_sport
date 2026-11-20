use leptos::{prelude::*};
use crate::reaction::SoundTimerComponent;
use crate::reaction::exercises::ExerciseBanner;

#[component]
pub fn DoBySignal() -> impl IntoView{
    view! {
        <SoundTimerComponent>
            <ExerciseBanner exercise_id=1001/>
        </SoundTimerComponent>
    }
}

