use leptos::{prelude::*};
use neuro_sport_ui::{
    navbar::Navbar, 
    sounds::SoundTimerComponent,
    auth::{Login, Register, AuthContext},
};
use leptos_router::{path, components::{Router, Routes, Route, Outlet, ParentRoute}};
use leptos_router::hooks::use_navigate;


#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    let auth = use_context::<AuthContext>()
        .expect("AuthContext должен быть предоставлен на уровне выше");
    
    let navigate = use_navigate();

    Effect::new(move |_| {
        if !auth.is_authenticated() {
            _ = navigate("/login", Default::default());
        }
    });

    Effect::new(move |_| {
        let handle = set_interval_with_handle(move || {
            if !auth.is_authenticated() {
                log::info!("Сессия истекла по фоновому таймеру");
            }
        }, std::time::Duration::from_secs(60));

        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });
    let children_fn = move || children();
    view! {
        <Show
            when=move || auth.session.get().is_some()
            fallback=move || view! { <div class="loading-placeholder">"Ожидание авторизации..."</div> }
        >
            {children_fn()}
        </Show>
    }
}


#[component]
fn App() -> impl IntoView{
    let context = AuthContext { session: RwSignal::new(None) };
    context.try_auto_login();

    provide_context(context);
    
    view! {
        <div class="app-container">
            <Router>
                <Navbar/>
                <main class="main-content">
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/register") view=Register/>
                        <Route path=path!("/login") view=Login/>    
                        <ParentRoute path=path!("")  view=|| view! { <AppLayout> <Outlet/> </AppLayout> }>
                            <Route path=path!("/") view=SoundTimerComponent/>
                        </ParentRoute>
                        <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> }/>
                    </Routes>
                </main>
            </Router>
        </div>
    }
} 

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
