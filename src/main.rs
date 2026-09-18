use leptos::{prelude::*};
use neuro_sport_ui::{navbar::Navbar, sounds::SoundTimerComponent};
use leptos_router::{path, components::{Router, Routes, Route}};


#[component]
fn App() -> impl IntoView{
    view! {
        <Router>
            <Navbar/>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=SoundTimerComponent/>
                    <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> }/>
                </Routes>
            </main>
        </Router>
    }
} 



fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
