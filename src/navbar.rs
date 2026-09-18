use leptos::{prelude::*};

#[component]
pub fn navbar() -> impl IntoView{
    
    view!{
        <nav class="navbar">
        <ul class="nav-links">
            <li><a href="/">Домой</a></li>
            <li><a href="sound">Статистика</a></li>
        </ul>
        </nav>
    }

} 
