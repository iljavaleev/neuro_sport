use leptos::{prelude::*};
use crate::auth::AuthContext;

#[component]
pub fn navbar() -> impl IntoView{
    let auth_context = use_context::<AuthContext>().expect("Context to be present");
    let show_logout = RwSignal::new(auth_context.is_authenticated());
    Effect::new(move |_| {
        show_logout.update(|show| *show = 
            auth_context.session.with(|s| s.is_some()));
    });


    view!{
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
                    
                   
                    {move || if show_logout.get(){
                            Some(view! {<a href="/logout" class="nav-link">"Выход"</a>})
                        }else{
                            Some(view!{<a href="/login" class="nav-link ms-auto">"Вход"</a>})
                        }
                    }
                </div>
            
            </div>
        </nav>
    }

} 
