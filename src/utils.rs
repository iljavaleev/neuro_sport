use leptos::prelude::*;
use leptos::ev::keydown;


pub fn on_key_down<F>(f: F)
where F: Fn() + 'static,
{
    let _ = window_event_listener(keydown, move |ev| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            f();
        }
    });
}
