use leptos::{prelude::*};

#[component]
pub fn navbar() -> impl IntoView{
    
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
                </div>
            
            </div>
        </nav>
    }

} 
