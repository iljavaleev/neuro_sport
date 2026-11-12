use serde::{Deserialize, Serialize};
use leptos::{prelude::*};
use chrono::NaiveDate;
use wasm_bindgen_futures::spawn_local;

use serde_wasm_bindgen::{from_value, to_value};
use leptos::logging::{error}; 

use crate::auth::utils::{checker, invoke};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SessionData {
    pub user_id: String,
    pub token: String,
    pub expires_at: f64,
}


#[derive(Clone, Copy)]
pub struct AuthContext {
    pub session: RwSignal<Option<SessionData>>, 
}


impl AuthContext {
    pub fn is_authenticated(&self) -> bool {
        if let Some(current_session) = self.session.get() {
            let now = js_sys::Date::now();
            
            if now < current_session.expires_at {
                return true;
            }
            self.logout();
        }
        false
    }

    pub fn get_user_id(&self) -> Option<String> {
        if self.is_authenticated() {
            self.session.get().map(|s| s.user_id)
        } else {
            None
        }
    }

    pub fn login(&self, valid_session: SessionData) {
        self.session.set(Some(valid_session.clone()));
        self.save_to_storage(&valid_session);
    }

    pub fn logout(&self) {
        self.session.set(None);
        if let Some(storage) = web_sys::window()
            .and_then(|w| 
                w
                .local_storage()
                .ok()
                .flatten()) {
            let _ = storage.remove_item("current_session");
        }
    }

     pub fn try_auto_login(&self) {
        let auth = *self; 

        spawn_local(async move {
            if let Some(local_data) = auth.load_from_storage() {
                #[derive(Serialize)]
                struct TokenArgs { token: String }

                let payload = TokenArgs { token: local_data.token };
                let args = to_value(&payload).unwrap();
            
                match invoke("verify_user_token", args).await{
                    Ok(js_session) => {
                        let session:Option<SessionData> = from_value(js_session).unwrap();
                        if session.is_some(){ // exists
                            auth.session.set(Some(session.unwrap()));
                            return;
                        }
                    }
                    Err(e) => {
                        let err = e.as_string().unwrap_or("".to_string());
                        error!("{:?}", err);
                    }
                }

            }
    
            auth.logout();
        });
    }

    
    fn save_to_storage(&self, session: &SessionData) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(json) = serde_json::to_string(session) {
                let _ = storage.set_item("current_session", &json);
            }
        }
    }


    fn load_from_storage(&self) -> Option<SessionData> {
        let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten())?;
        let json = storage.get_item("current_session").ok().flatten()?;
        serde_json::from_str(&json).ok()
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
        checker::check_password(&self.password)
    }

    fn check_date_of_birth(&self) -> Result<(), String>{
        let dob = match self.date_of_birth {
            Some(date) => date,
            None => return Err("Пожалуйста, укажите дату рождения".to_string()),
        };

        checker::check_date_of_birth(dob)
    }

    fn check_email(&self) -> Result<(), String>{
        checker::check_email(&self.email)
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
pub struct CreateUserArg {
    pub user: User, 
}

#[derive(Serialize)]
pub struct CheckUserEmailArg {
    pub email: String, 
}


#[derive(Deserialize, Debug, Default)]
pub struct UserResponse {
    pub id: String, 
    pub email: String,
}


#[derive(Serialize)]
pub struct CheckUserPasswArg {
    pub id: String,
    pub passwd: String, 
}

#[derive(Serialize)]
pub struct UpdateUserPasswArg {
    pub code: String,
    pub newPassword: String, 
}


#[derive(Copy, Clone)]
pub struct LoginContext{
    pub db_user: RwSignal<UserResponse>,
    pub msg_str: RwSignal<String>,
}


#[derive(Serialize, Deserialize)]
pub struct EmailArgs { 
    pub email: String 
}

#[derive(Serialize, Deserialize)]
pub struct VerifyArgs { 
    pub email: String, 
    pub code: String 
}

#[derive(Serialize, Deserialize, Default)]
pub struct OtpJsResponse {
    pub success: bool,
    pub message: String,
    pub code: Option<String>,
    pub auth_info: Option<SessionData>,
}
