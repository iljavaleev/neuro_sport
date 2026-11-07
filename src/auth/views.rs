use leptos::{prelude::*};
use chrono::{NaiveDate};
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen_futures::spawn_local;
use log::{info, error}; 
use crate::auth::types::*;
use crate::auth::utils::checker;



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

#[component]
pub fn CheckEmailView (
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

// написать хэндлеры
#[component]
pub fn UpdatePasswDbView(
    #[prop(into)] 
    forgot_passwd: RwSignal<bool>,
    #[prop(into)] 
    frontend_code_for_otp: RwSignal<Option<String>>,
    #[prop(into)]
    otp_verified: RwSignal<bool>
    ) -> impl IntoView{
    let new_pass = RwSignal::new(Passw::default());
    let LoginContext{db_user: _, err_str} 
        = use_context::<LoginContext>().expect("Context to be present");

    if !frontend_code_for_otp.get().is_some(){
        err_str.update(|e| *e = "Ошибка верификации".to_string() );
        otp_verified.update(|v| *v = false );
    }

    let submit_update_pass = move |_| {
        let err_str = err_str;
        let passwd = new_pass.with(|p| p.pass.clone());
        
        if let Err(err) = checker::check_password(&passwd) {
            err_str.update(|e| *e = err); 
            return
        }

        spawn_local(async move {
            let payload = UpdateUserPasswArg {
                code: frontend_code_for_otp.get_untracked().unwrap(),
                newPassword: passwd
            };
            
            if let Ok(args) = to_value(&payload) {
                match invoke("update_user_password", args).await {
                    Ok(_) => {
                        forgot_passwd.update(|p| *p = false)
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
                    Ok(res_js) => {
                        let res: OtpJsResponse = serde_wasm_bindgen::from_value(res_js).unwrap();
                        if res.success{
                            // todo login and to main
                        }
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
    
    let otp_verified = RwSignal::new(false);
    let frontend_code_for_otp = RwSignal::new(Some(String::new()));
    
    
    view!{
        <Show 
            when=move || forgot_passwd.get()  
            fallback=move || view! { <CheckPasswDbView forgot_passwd=forgot_passwd/> }
            >
            <Show
                when=move || otp_verified.get()
                fallback={
                    move || view! { 
                        <ForgotPasswordView 
                            otp_verified=otp_verified
                            frontend_code_for_otp=frontend_code_for_otp 
                        /> 
                    }
                }
            >
                <UpdatePasswDbView 
                    forgot_passwd=forgot_passwd
                    frontend_code_for_otp=frontend_code_for_otp
                    otp_verified=otp_verified
                />
            </Show>
        </Show>

    }
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

#[component]
pub fn ForgotPasswordView(
    #[prop(into)] 
    otp_verified: RwSignal<bool>,
    #[prop(into)] 
    frontend_code_for_otp: RwSignal<Option<String>>
) -> impl IntoView {
    
    let LoginContext{db_user, err_str} 
        = use_context::<LoginContext>().expect("Context to be present");
   

    let code = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let code_sent = RwSignal::new(false);

    let verified_email = db_user.with(|u| u.email.clone());

    let email_current = verified_email.clone();
    let email_for_verify = verified_email.clone();


    let request_otp_action = Action::new_local(move |email_str: &String| {
        let e = email_str.clone();
        async move {
            let args = serde_wasm_bindgen::to_value(&EmailArgs { email: e }).unwrap();
            if let Ok(res_js) = invoke("request_password_reset_otp", args).await {
                let res: OtpJsResponse = serde_wasm_bindgen::from_value(res_js).unwrap();
                res
            } else {
                OtpJsResponse { 
                    success: false, 
                    message: "Ошибка связи с бэкендом".to_string(),
                    code: None 
                }
            }
        }
    });


    let verify_otp_action = Action::new_local(move |(email_str, code_val): &(String, String)| {
        let e = email_str.clone();
        let c = code_val.clone();
        async move {
            let args = serde_wasm_bindgen::to_value(&VerifyArgs { email: e, code: c }).unwrap();
            if let Ok(res_js) = invoke("verify_password_reset_otp", args).await {
                let res: OtpJsResponse = serde_wasm_bindgen::from_value(res_js).unwrap();
                res
            } else {
                OtpJsResponse { 
                    success: false, 
                    message: "Ошибка верификации".to_string(),
                    code: None
                }
            }
        }
    });

    
    Effect::new(move |_| {
        request_otp_action.value().with(|val| {
            if let Some(res) = val {
                message.set(res.message.clone());
                if res.success {
                    code_sent.set(true);
                }
            }
        });
    });

    Effect::new(move |_| {
        verify_otp_action.value().with(|val| {
            if let Some(res) = val {
                message.set(res.message.clone());
                if res.success{
                    frontend_code_for_otp.update(|c| *c = res.code.clone());
                    otp_verified.update(|v| *v = true );
                }else {
                    err_str.update(|e| *e = res.message.clone() )
                }
            }
        });
    });

    let is_loading = move || request_otp_action.pending().get() || verify_otp_action.pending().get();

    view! {
        <div style="padding: 2rem; max-width: 25rem; margin: 0 auto; font-size: 1rem;">
            <h2>"Восстановление доступа"</h2>
            
            <p style="color: #555; font-size: 0.9rem; margin-bottom: 1.5rem;">
                {move || if code_sent.get() { 
                    format!("Код отправлен на {}", verified_email) 
                } else { 
                    format!("Мы отправим код подтверждения на почту {}", verified_email) 
                }}
            </p>

            <Show
                when=move || code_sent.get()
                fallback=move || {
                    let email_current = email_current.clone();
                    view! {
                        <div style="display: flex; flex-direction: column; gap: 0.8rem;">
                            <button 
                                style="padding: 0.7rem; font-size: 1rem; background-color: #0076ff; color: white; border: none; border-radius: 0.25rem; cursor: pointer;"
                                prop:disabled=is_loading
                                on:click=move |_| { request_otp_action.dispatch(email_current.clone()); }>
                                {move || if request_otp_action.pending().get() { "Отправка кода..." } else { "Получить код восстановления" }}
                            </button>
                        </div>
                    }
                }
            >
                {
                    let email_for_verify = email_for_verify.clone();
                    view! {
                        <div style="display: flex; flex-direction: column; gap: 0.8rem;">
                            <input 
                                type="text" 
                                placeholder="000000"
                                maxlength="6"
                                style="padding: 0.6rem; font-size: 1.2rem; text-align: center; letter-spacing: 0.3rem; border-radius: 0.25rem; border: 0.05rem solid #ccc;"
                                on:input=move |ev| code.set(event_target_value(&ev))
                                prop:value=code
                                prop:disabled=is_loading
                            />
                            <button 
                                style="padding: 0.7rem; font-size: 1rem; background-color: #34c759; color: white; border: none; border-radius: 0.25rem; cursor: pointer;"
                                prop:disabled=is_loading
                                on:click=move |_| { verify_otp_action.dispatch((email_for_verify.clone(), code.get())); }>
                                {move || if verify_otp_action.pending().get() { "Проверка..." } else { "Подтвердить код" }}
                            </button>
                        </div>
                    }
                }
            </Show>

            <p style="margin-top: 1rem; color: #d10000; font-size: 0.85rem; text-align: center;">
                {move || message.get()}
            </p>
        </div>
    }
}
