
use std::{sync::LazyLock};
use regex::Regex;
use chrono::{Local, NaiveDate, Datelike};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub mod checker{
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