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

pub mod Checker{
    use super::*;
    pub fn check_password(password: &String) -> Result<(), String>{
            
        if password.len() < 8 {
            return Err("Пароль должен быть не менее 8 символов в длину".to_string());
        }
        
        if password.len() > 64 {
            return Err("Пароль слишком длинный (максимум 64 символа)".to_string());
        }
        
        let mut has_digit = false;
        let mut has_special = false;

        for ch in password.chars() {
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

    pub fn check_date_of_birth(date_of_birth: NaiveDate) -> Result<(), String>{
        
        let today = Local::now().naive_local().date();

        if date_of_birth > today {
            return Err("Дата рождения не может быть в будущем".to_string());
        }

        let mut age = today.year() - date_of_birth.year();
        
        if today.month() < date_of_birth.month() || (today.month() == date_of_birth.month() && today.day() < date_of_birth.day()) {
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

    pub fn check_email(email: &String) -> Result<(), String>{
        if email.trim().is_empty() {
            return Err("Email не может быть пустым".to_string());
        }

        static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
        });

        if !EMAIL_REGEX.is_match(email.as_str()) {
            return Err("Некорректный формат Email адреса".to_string());
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        Checker::check_password(&self.password)
    }

    fn check_date_of_birth(&self) -> Result<(), String>{
        let dob = match self.date_of_birth {
            Some(date) => date,
            None => return Err("Пожалуйста, укажите дату рождения".to_string()),
        };

        Checker::check_date_of_birth(dob)
    }

    fn check_email(&self) -> Result<(), String>{
        Checker::check_email(&self.email)
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

#[derive(Serialize)]
struct CreateUserArg {
    user: User, 
}

#[component]
pub fn Register() -> impl IntoView{
    let user_info = RwSignal::new(User::new());
    let passwords = RwSignal::new(Passw::default());
    let err_str =  RwSignal::new(String::new());

    let navigate = use_navigate();
    let view_nav = navigate.clone();
    
    let exit_button = move |_| {
        
        if passwords.with(|e| e.pass != e.pass2){
            err_str.set("Пароли не совпадают".to_string());
            return;
        }
        user_info.update(|u| 
            u.password = passwords.with(|p| p.pass.clone())
        );

        let mut has_error = false;
        user_info.with(|u| {
            if let Err(e) = u.check() {
                err_str.set(e);
                has_error = true;
            }
        });

        if has_error {
            return;
        }
        let navigate = use_navigate();

        spawn_local(async move {
            let payload = CreateUserArg { user: user_info.get() };
            let args = to_value(&payload).unwrap();
            
            if let Err(err) = invoke("create_user", args).await{
            {
                let err_msg: String = from_value(err)
                        .unwrap_or_else(|_| "Unknown backend error".to_string());
                error!("{}", err_msg);
                err_str.set(err_msg);
            }}else{
                navigate("/", NavigateOptions::default());
            }
        });

        
    };

    view!{
        <main class="main-content auth-page">
        <div class="auth-top-wrapper">
            <div class="panel-card auth-card" style="width: 100%; max-width: 460px;">
                <h2>"Регистрация"</h2>
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
                        type="password"

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
                        type="password"

                        prop:value=move || (*passwords.read()).pass2.clone()
                        
                        on:input:target=move |ev| {
                            (*passwords.write()).pass2 = 
                            ev.target().value().parse().unwrap();
                        }
                    />
                </div>

                <div class="input-row-two">
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
                </div>
                    <div>
                        {move || {
                            let error = err_str.get();
                            (!error.is_empty()).then(|| view! { 
                                <div class="task-banner" style="margin-bottom: 1rem; border-color: rgba(239, 68, 68, 0.4); background-color: rgba(239, 68, 68, 0.1); color: #f87171;">
                                    <strong>"Ошибка: "</strong> {error}
                                </div> 
                            })
                        }}
                        <div class="form-footer-actions">
                            <button type="submit" class="btn-primary" on:click=exit_button>
                                "Завершить"
                            </button>
                            
                            <button class="action-link" on:click=move |_| view_nav("/login", NavigateOptions::default())>
                                "назад"
                            </button>
                        </div>
                    </div>
                </div>
            
        </div>
        </main>
    }

}



#[derive(Serialize)]
struct CheckUserEmailArg {
    email: String, 
}


#[derive(Deserialize, Debug, Default)]
pub struct UserResponse {
    pub id: String, 
    pub email: String,
}

#[component]
pub fn CheckEmailView(
    #[prop(into)] 
    user_exists: RwSignal<bool>,
    ) -> impl IntoView{
    let LoginContext{db_user, err_str} = use_context::<LoginContext>().expect("Context to be present");
    let navigate = use_navigate();

    let continue_btn = move |_| {
        let email = db_user.with(|u| u.email.clone());
        let user_exists = user_exists;
        let err_str = err_str;

        
        spawn_local(async move {
            let payload = CheckUserEmailArg { email };
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
                    let err_msg: String = from_value(js_error)
                            .unwrap_or_else(|_| "Unknown backend error".to_string());
                    err_str.set(err_msg);
                }
            }
        });
    };

    view!{
        <main class="main-content auth-page">
        <div class="auth-top-wrapper">

            <div class="panel-card auth-card" style="width: 100%; max-width: 440px;">
                <h2>"Вход в систему"</h2>
                <div class="form-group">
                    <label>"Введите адрес эл.почты"</label>
                    <input 
                        type="email"
                        placeholder="name@example.com"

                        prop:value=move || db_user.with(|u| u.email.clone())
                        
                        on:input:target=move |ev| {
                            db_user.update(|u| u.email = ev.target().value());
                        }
                    />
                </div>
            
                {move || {
                    let error = err_str.get();
                    (!error.is_empty()).then(|| view! { 
                        <div class="task-banner" style="margin-bottom: 1rem; border-color: rgba(239, 68, 68, 0.4); background-color: rgba(239, 68, 68, 0.1); color: #f87171;">
                                <strong>"Ошибка: "</strong> {error}
                        </div> 
                    })
                }}
                <div class="form-footer-actions">
                    <button class="btn-primary" on:click=continue_btn>
                        "Продолжить"
                    </button>
                    
                    <button class="action-link" on:click=move |_| navigate("/register", NavigateOptions::default())>
                        "создать аккаунт?"
                    </button>
                </div>
                </div>
            </div>
        </main>
    }
    
}



#[derive(Serialize)]
struct CheckUserPasswArg {
    id: String,
    passwd: String, 
}

// написать хэндлеры

#[component]
pub fn UpdatePasswDbView(
     #[prop(into)] 
    forgot_passwd: RwSignal<bool>) -> impl IntoView{
    let new_pass = RwSignal::new(Passw::default());
    let LoginContext{db_user, err_str} = use_context::<LoginContext>().expect("Context to be present");


    let submit_update_pass = move |_| {
        let id = db_user.with(|u| u.id.clone());
        let err_str = err_str;
        let passwd = new_pass.with(|p| p.pass.clone());
        
        if let Err(err) = Checker::check_password(&passwd) {
            err_str.update(|e| *e = err); 
            return
        }

        spawn_local(async move {
            let payload = CheckUserPasswArg {
                id,
                passwd
            };
            
            if let Ok(args) = to_value(&payload) {
                match invoke("update_user_password", args).await {
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

    view! {
        <main class="main-content auth-page">
        <div class="auth-top-wrapper">
            <div class="panel-card auth-card" style="width: 100%; max-width: 440px;">
                <h2>"Сброс пароля"</h2>
                <div class="form-group" style="margin-bottom: 1rem;">
                    <label class="form-label">"Введите новый пароль"</label>
                    <input 
                        type="text"

                        prop:value=move || (*new_pass.read()).pass.clone()
                        
                        on:input:target=move |ev| {
                            new_pass.update(|p| p.pass = ev.target().value());
                        }
                    />
                </div>
                <div class="form-group" style="margin-bottom: 1.5rem;">
                    <label class="form-label">"Подтвердите пароль"</label>
                    <input 
                        type="text"

                        prop:value=move || (*new_pass.read()).pass2.clone()
                        
                        on:input:target=move |ev| {
                            new_pass.update(|p| p.pass2 = ev.target().value());
                        }
                    />
                </div>
                <div class="form-footer-actions">
                    {move || {
                        let error = err_str.get();
                        (!error.is_empty()).then(|| view! { 
                            <div class="task-banner" style="margin-bottom: 1rem; border-color: rgba(239, 68, 68, 0.4); background-color: rgba(239, 68, 68, 0.1); color: #f87171; width: 100%; text-align: center;">
                                <strong>"Ошибка: "</strong> {error}
                            </div> 
                        })
                    }}
                    
                    <button class="btn-primary" on:click=submit_update_pass>
                        "Сохранить и войти"
                    </button>

                    <button 
                        class="action-link" 
                        on:click=move |_| forgot_passwd.update(|p| *p = false)
                    >
                        "назад?"
                    </button>
                </div>
            </div>
        </div>
        </main>
    }
}


#[component]
pub fn CheckPasswDbView(
    #[prop(into)] 
    forgot_passwd: RwSignal<bool>) -> impl IntoView{
    let LoginContext{db_user, err_str} = use_context::<LoginContext>().expect("Context to be present");

    let password = RwSignal::new(String::new());
    let submit_check_pass = move |_| {
        let id = db_user.with(|u| u.id.clone());
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


    view!{

        <main class="main-content auth-page">
        <div class="auth-top-wrapper">
            <div class="panel-card auth-card"> 
                <h2>"Вход в систему"</h2>
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
                
                <div class="form-footer-actions">
                    {move || {
                        let error = err_str.get();
                        (!error.is_empty()).then(|| view! { 
                            <div class="text-red-500 font-bold bg-red-100 p-2 rounded">
                                {error}
                            </div> 
                        })
                    }}
                    <button class="btn-primary" on:click=submit_check_pass>
                        "Вход"
                    </button>
                
                    <button 
                        class="action-link" 
                        on:click=move |_| forgot_passwd.update(|p| *p = true)
                    >
                        "забыли пароль?"
                    </button>
                    // добавить назад
                </div>
            </div>
        </div>
        </main>
    }
}



#[component]
pub fn CheckPasswView() -> impl IntoView{
    let forgot_passwd = RwSignal::new(false);
    view!{
        <Show 
            when=move || forgot_passwd.get()  
            fallback=move || view! { <CheckPasswDbView forgot_passwd=forgot_passwd/> }
            >
            <UpdatePasswDbView forgot_passwd=forgot_passwd/>
        </Show>

    }
}


#[derive(Copy, Clone)]
struct LoginContext{
    db_user: RwSignal<UserResponse>,
    err_str: RwSignal<String>,
}

#[component]
pub fn Login() -> impl IntoView{
    let user_exists = RwSignal::new(false);
    
    provide_context(LoginContext{
        db_user: RwSignal::new(UserResponse::default()),
        err_str: RwSignal::new(String::default())
    });
    
    view! {
        <div>
            <Show 
                when=move || user_exists.get()  
                fallback=move || view! { <CheckEmailView user_exists=user_exists/> }
            >
                <CheckPasswView />
            </Show>
        </div>
    }
}
