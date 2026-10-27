use leptos::{prelude::*};
use serde::{Deserialize, Serialize};
use chrono::{Local, NaiveDate, Datelike};
use regex::Regex;
use std::sync::LazyLock;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen_futures::spawn_local;
use log::{info, error}; 


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}


#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub password: String,
    pub date_of_birth: Option<NaiveDate>,
    pub sex: String,
}

impl User {
    pub fn new() -> Self{
        Self{
            email: String::new(),
            password: String::new(),
            date_of_birth: None,
            sex: String::new(),
        }
    }

    fn check_password(&self) -> Result<(), String>{
        
        if self.password.len() < 8 {
            return Err("Пароль должен быть не менее 8 символов в длину".to_string());
        }
        
        if self.password.len() > 64 {
            return Err("Пароль слишком длинный (максимум 64 символа)".to_string());
        }
        
        let mut has_digit = false;
        let mut has_special = false;

        for ch in self.password.chars() {
            if ch.is_ascii_digit() {
                has_digit = true;
            } else if ch.is_ascii_punctuation() || "§±!@#$%^&*()_+-=[]{}|;':\",./<>?".contains(ch) {
                has_special = true;
            }
        }

        if !has_digit {
            return Err("Пароль должен содержать хотя бы одну цифру".to_string());
        }
        if !has_special {
            return Err("Пароль должен содержать хотя бы один специальный символ (например, !, @, #, $)".to_string());
        }

        Ok(())
    }

    fn check_date_of_birth(&self) -> Result<(), String>{
        let dob = match self.date_of_birth {
            Some(date) => date,
            None => return Err("Пожалуйста, укажите дату рождения".to_string()),
        };

        let today = Local::now().naive_local().date();

        if dob > today {
            return Err("Дата рождения не может быть в будущем".to_string());
        }

        let mut age = today.year() - dob.year();
        
        if today.month() < dob.month() || (today.month() == dob.month() && today.day() < dob.day()) {
            age -= 1;
        }

        if age < 14 {
            return Err("Регистрация доступна только пользователям от 14 лет".to_string());
        }
        if age > 90 {
            return Err("Указан нереалистичный возраст".to_string());
        }
        
        Ok(())
    }

    fn check_email(&self) -> Result<(), String>{
        if self.email.trim().is_empty() {
            return Err("Email не может быть пустым".to_string());
        }

        static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
        });

        if !EMAIL_REGEX.is_match(self.email.as_str()) {
            return Err("Некорректный формат Email адреса".to_string());
        }
        
        Ok(())
    }

    pub fn check(&self) -> Result<(), String>{
        if let Err(err) = self.check_password() {
            return Err(err);
        }
        if let Err(err) = self.check_email() {
            return Err(err);
        }
        if let Err(err) = self.check_date_of_birth() {
            return Err(err);
        }

        Ok(())
    }
}


#[derive(Default)]
pub struct Passw{
    pub pass: String,
    pub pass2: String
}


#[component]
pub fn register() -> impl IntoView{
    let user_info = RwSignal::new(User::new());
    let passwords = RwSignal::new(Passw::default());
    let err_str =  RwSignal::new(String::new());
    let navigate = use_navigate();

    let exit_button = move |_| {
        
        if passwords.with(|e| e.pass != e.pass2){
            *err_str.write() = "Пароли не совпадают".to_string();
            return;
        }
        user_info.update(|u| 
            u.password = passwords.with(|p| p.pass.clone())
        );

        user_info.with(|u|{
            if let Err(e) = u.check(){
                 *err_str.write() = e;
            }
            return;
        });
        navigate("/", NavigateOptions::default());
    };

    view!{
        <div>
            <div class="form-group">
                <label>"Адрес эл.почты"</label>
                <input 
                    type="text"

                    prop:value=move || (*user_info.read()).email.clone()
                    
                    on:input:target=move |ev| {
                        (*user_info.write()).email = 
                        ev.target().value().parse().unwrap();
                    }
                />
            </div>
            <div class="form-group">
                <label>"Пароль"</label>
                <input 
                    type="text"

                    prop:value=move || (*passwords.read()).pass.clone()
                    
                    on:input:target=move |ev| {
                        (*passwords.write()).pass = 
                        ev.target().value().parse().unwrap();
                    }
                />
            </div>
            <div class="form-group">
                <label>"Повторите пароль"</label>
                <input 
                    type="text"

                    prop:value=move || (*passwords.read()).pass2.clone()
                    
                    on:input:target=move |ev| {
                        (*passwords.write()).pass2 = 
                        ev.target().value().parse().unwrap();
                    }
                />
            </div>
            <div class="form-group">
                <label>"Дата рождения"</label>
                <input 
                    type="date"

                    prop:value=move || {
                        user_info.with(|info| {
                            info.date_of_birth
                                .map(|date| date.to_string()) 
                                .unwrap_or_default() 
                        })
                    }
                    
                    on:input:target=move |ev| {
                        let val: String =  ev.target().value().parse().unwrap();
                        
                        user_info.update(|info| {
                            info.date_of_birth = if val.is_empty() {None} 
                                else 
                            {NaiveDate::parse_from_str(&val, "%Y-%m-%d").ok()};
                        });
                    }
                />
            </div>
            <div class="form-group">
                <label>"Пол"</label>
                <select
                        class="select"
                        on:change:target=move |ev| {
                            (*user_info.write()).sex = ev.target().value().parse().unwrap()
                        }
                        prop:value=move || (*user_info.read()).sex.clone()
                    >
                        <option value="M">"М"</option>
                        <option value="W">"Ж"</option>
                </select>
            </div>
            <div>
                {move || {
                    let error = err_str.get();
                    (!error.is_empty()).then(|| view! { 
                        <div class="text-red-500 font-bold bg-red-100 p-2 rounded">
                            {error}
                        </div> 
                    })
                }}
                <button class="btn-primary" on:click=exit_button>
                    <p>"Завершить"</p>
                </button>
            </div>
        </div>
    }

}



#[derive(Serialize)]
struct CheckUserEmailArg {
    email: String, 
}


#[derive(Deserialize, Debug, Default)]
pub struct UserResponse {
    pub id: u64, 
    pub email: String,
}

#[component]
pub fn CheckEmailView(
    #[prop(into)] 
    user_exists: RwSignal<bool>,
    #[prop(into)]
    db_user: RwSignal<UserResponse>) -> impl IntoView{
    
    let email = RwSignal::new(String::new());
    let err_str =  RwSignal::new(String::new());
    
    let continue_btn = move |_| {
        let email = email;
        let user_exists = user_exists;
        let err_str = err_str;

        
        spawn_local(async move {
            let payload = CheckUserEmailArg {
                email: email.get_untracked(), 
            };
            let args = to_value(&payload).unwrap();
            
            match invoke("check_user_exists", args).await{
                Ok(js_result) => {
                    match from_value(js_result){
                        Ok(user) => db_user.set(user),
                        Err(_) => *err_str.write() = "Unknown backend error".to_string(),
                    }
                    user_exists.set(true)
                },
                Err(js_error) => {
                    *err_str.write()  = serde_wasm_bindgen::from_value(js_error)
                        .unwrap_or_else(|_| "Unknown backend error".to_string());
                }
            }
        });
    };

    view!{
        <div>
            <div class="form-group">
                <label>"Введите адрес эл.почты"</label>
                <input 
                    type="text"

                    prop:value=move || email.get()
                    
                    on:input:target=move |ev| {
                        email.set(ev.target().value());
                    }
                />
            </div>
            <div>
                {move || {
                    let error = err_str.get();
                    (!error.is_empty()).then(|| view! { 
                        <div class="text-red-500 font-bold bg-red-100 p-2 rounded">
                            {error}
                        </div> 
                    })
                }}
                <button class="btn-primary" on:click=continue_btn>
                    <p>"Продолжить"</p>
                </button>
            </div>
            <div class="nav-links">
                <a href="/auth" class="nav-link active">
                    "создать аккаунт"
                </a>
            </div>
        </div>
    }
    
}



#[derive(Serialize)]
struct CheckUserPasswArg {
    id: u64,
    passwd: String, 
}


#[component]
pub fn CheckPasswView(
     #[prop(into)] 
    user_id: RwSignal<UserResponse>) -> impl IntoView{
    let password = RwSignal::new(String::new());
    let err_str =  RwSignal::new(String::new());

    let forgot_passwd = RwSignal::new(false);
    
    let submit_btn = move |_| {
        let id = user_id.with(|u| u.id);
        let password = password;
        let err_str = err_str;

        
        spawn_local(async move {
            let payload = CheckUserPasswArg {
                id,
                passwd: password.get_untracked(), 
            };
            
            if let Ok(args) = to_value(&payload) {
                match invoke("check_user_password", args).await {
                    Ok(_) => {
                        // Handle successful login here (e.g., redirect or update state)
                    },
                    Err(js_error) => {
                        let err_msg = serde_wasm_bindgen::from_value(js_error)
                            .unwrap_or_else(|_| "Unknown backend error".to_string());
                        err_str.set(err_msg);
                    }
                }
            }
        });
    };

    // add show for forgot password
    // <Show 
    //             when=move || user_exists.get()  
    //             fallback=move || view! { <CheckEmailView user_exists=user_exists db_user=user_resp/> }
    //         >
    //             <CheckPasswView user_id=user_resp />
    //         </Show>

    view!{
        <div>
            <div class="form-group">
                <label>"Введите пароль"</label>
                 <input 
                    type="text"

                    prop:value=move || password.get()
                    
                    on:input:target=move |ev| {
                        password.set(ev.target().value());
                    }
                />
            </div>
            
            <div>
                {move || {
                    let error = err_str.get();
                    (!error.is_empty()).then(|| view! { 
                        <div class="text-red-500 font-bold bg-red-100 p-2 rounded">
                            {error}
                        </div> 
                    })
                }}
                <button class="btn-primary" on:click=submit_btn>
                    <p>"Вход"</p>
                </button>
            </div>

            <div class="nav-links">
                <a href="/auth" class="nav-link active">
                    "Забыли пароль?"
                </a>
            </div>
        </div>
    }
    
}




#[component]
pub fn login() -> impl IntoView{
    let user_exists = RwSignal::new(false);
    let user_resp = RwSignal::new(UserResponse::default());

    let user_info = RwSignal::new(User::new());
    Effect::new(move |_| {
        if user_exists.get() {
            let email = user_resp.with(|u| u.email.clone());
            if !email.is_empty() {
                user_info.update(|u| u.email = email);
            }
        }
    });

    view! {
        <div>
            <Show 
                when=move || user_exists.get()  
                fallback=move || view! { <CheckEmailView user_exists=user_exists db_user=user_resp/> }
            >
                <CheckPasswView user_id=user_resp />
            </Show>
        </div>
    }
}


// <Show when=move || buttons_visible.get() 
                    
//     fallback=move || view! { 
//         <div class="timer-controls-row" style="grid-template-columns: 1fr;">
//             <button class="btn-primary" on:click=move |_| { 
//                 try_handle.get().unwrap().clear();
//                 pause_round();
//                 *buttons_visible.write() = true
//             }>
//                 "Пауза"
//             </button>
//         </div>
//     }>
    
//     <div class="timer-controls-row">
//         <button class="btn-primary" on:click=move |_| { 
//             resume_timer(); 
//             resume_round();
//             *buttons_visible.write() = false 
//         } >
//             "Продолжить"
//         </button>
//         <button class="btn-secondary" on:click=move |_| {
//             abort_round();
//             reset_timer();
//             play_round_timer();
//             resume_timer();
//             *buttons_visible.write() = false 
//         } >
//             "Заново"
//         </button>
//     </div>
//     <button class="back-link" on:click=move |_| { 
//             (*timer_context.write()).view_timer = false;
//             try_handle.get().unwrap().clear();
//             abort_round();
//         } >
//             "Вернуться к настройкам"
//     </button>
// </Show>