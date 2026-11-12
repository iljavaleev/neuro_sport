use leptos::{prelude::*};
use crate::auth::AuthContext;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

#[component]
pub fn navbar() -> impl IntoView{
    let auth_context = use_context::<AuthContext>()
        .expect("Context to be present");
        
    let show_logout = RwSignal::new(auth_context.is_authenticated());
    
    Effect::new(move |_| {
        show_logout.update(|show| *show = 
            auth_context.session.with(|s| s.is_some()));
    });


    let handle_logout = move |_| {
        auth_context.logout();
        
        let navigate = use_navigate();
        navigate("/login", NavigateOptions::default());
    };

    view! {
        <nav class="navbar">
            <div class="navbar-content">
                
                <div class="nav-brand">
                    "SignalTracker"
                </div>
                
                <div class="nav-links">
                    <a href="/" class="nav-link active">
                        "Домой"
                    </a>
                    
                    <a href="/sound" class="nav-link">
                        "Статистика"
                    </a>
                    
                   {move || if show_logout.get() {
                        Some(view! {
                            <button 
                                class="nav-link style-as-link" 
                                on:click=handle_logout
                            >
                                "Выход"
                            </button>
                        }.into_any()) // Добавлено .into_view()
                    } else {
                        Some(view! {
                            <a href="/login" class="nav-link ms-auto">"Вход"</a>
                        }.into_any()) // Добавлено .into_view()
                    }}
                </div>
            
            </div>
        </nav>
    }

} 
