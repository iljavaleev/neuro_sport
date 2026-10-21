use leptos::{prelude::*};
use serde::{Deserialize, Serialize};
use chrono::{Local, NaiveDate, Datelike};
use regex::Regex;
use std::sync::OnceLock;


pub fn validate_email(email: &str) -> Result<(), String> {
    if email.trim().is_empty() {
        return Err("Email не может быть пустым".to_string());
    }

    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .unwrap()
    });

    if !regex.is_match(email) {
        return Err("Некорректный формат Email адреса".to_string());
    }

    Ok(())
}

pub fn validate_age(date_of_birth: Option<NaiveDate>) -> Result<(), String> {
    
    let dob = match date_of_birth {
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

    pub fn check(&self) -> Result<(), String>{
        
        
        Ok(())
    }

}

pub struct Passw{
    pub pass: String,
    pub pass2: String
}

impl Passw{
    pub fn new() -> Self{
        Self{
            pass: String::new(),
            pass2: String::new(),
        }
    }

    pub fn check(&self) -> Result<(), String>{
        if self.pass != self.pass2{
            return Err("Пароли не совпадают".to_string());
        }
        
        if self.pass.len() < 8 {
            return Err("Пароль должен быть не менее 8 символов в длину".to_string());
        }
        
        if self.pass.len() > 64 {
            return Err("Пароль слишком длинный (максимум 64 символа)".to_string());
        }
        
        let mut has_digit = false;
        let mut has_special = false;

        for ch in self.pass.chars() {
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
}

#[component]
pub fn register() -> impl IntoView{
    let user_info = RwSignal::new(User::new());
    let passwords = RwSignal::new(Passw::new());

    let check = move ||{
        if let Err(err) = validate_email(&(*user_info.read()).email){
            return err;
        }
        
        if let Err(err) = validate_age((*user_info.read()).date_of_birth){
            return err;
        }
        // переделать
        if let Err(err) = (*passwords.read()).check(){
            return err;
        }else {
            (*user_info.write()).password = *&(passwords.read()).pass.to_string();    
        }

        let  = ;

    }

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
            <button class="btn-primary" on:click=move |_| {
               
            }>
                <p>"Завершить"</p>
            </button>
        </div>
    }

}