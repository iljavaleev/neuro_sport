use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2
};
use log::{info, error};
use tauri::State;
use crate::state::AppState;
use crate::types::{FrontendUser, UserResponse};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};


#[tauri::command]
pub async fn create_user(
    state: State<'_, AppState>,
    user: FrontendUser
) -> Result<(), String> {
 
    let new_id = uuid::Uuid::new_v4().to_string(); 
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(user.password.as_bytes())
        .map_err(|e| e.to_string())? 
        .to_string();
    
    sqlx::query!(
        r#"
        INSERT INTO user (id, email, password_hash, date_of_birth, sex)
        VALUES (?, ?, ?, ?, ?)
        "#,
        new_id,
        user.email,
        password_hash,
        user.date_of_birth,
        user.sex
    )
    .execute(&state.db)
    .await
    .map_err(|e| format!("Ошибка сохранения полльзователя в БД: {}", e))?;

    Ok(())
}


#[tauri::command]
pub async fn check_user_exists(
    state: State<'_, AppState>,
    email: String
) -> Result<UserResponse, String> {
    
    let row = sqlx::query!(
        "SELECT id, email FROM user WHERE email = ?",
        email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    let user_data = row.ok_or_else(|| "Пользователь не существует".to_string())?;

    Ok(UserResponse {
        id: user_data.id,
        email: user_data.email,
    })
}



#[tauri::command]
pub fn register_user(password_from_frontend: String) -> Result<(), String> {
    
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password_from_frontend.as_bytes())
        .map_err(|e| e.to_string())? 
        .to_string();
    
    
    // let parsed_hash = PasswordHash::new(&password_hash)?;
    // assert!(Argon2::default().verify_password(password, &parsed_hash).is_ok());

    // 4. Save `password_hash` to your SQLite table
    // execute_sql("INSERT INTO users (password) VALUES (?)", vec![password_hash]);

    Ok(())
}

use sqlx::SqlitePool;
use rand::RngExt;
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// https://mailtrap.io/sandboxes/4998701/settings
#[tauri::command]
async fn send_workout_report(recipient_email: String, body_text: String) -> Result<String, String> {
    let email = Message::builder()
        .from("Sport Timer App <no-reply@iljavaleev.com>".parse().unwrap())
        .to(recipient_email.parse().map_err(|e| format!("Invalid recipient: {e}"))?)
        .subject("Your Agility Workout Statistics")
        .body(body_text)
        .map_err(|e| e.to_string())?;

    // 2. Set up the relay server transport (e.g., Mailtrap, SendGrid, etc.)
   let creds = Credentials::new(
        "9ff9ba3764b2e1".to_string(), 
        "fa8f3aa6b0ce98".to_string()
    );
    
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("sandbox.smtp.mailtrap.io")
        .map_err(|e| e.to_string())?
        .port(2525)
        .credentials(creds)
        .build();

    match mailer.send(email).await {
        Ok(_) => Ok("Email sent successfully!".to_string()),
        Err(err) => Err(format!("Delivery failed (Check connectivity/settings): {err}")),
    }
}

pub struct DbState(pub SqlitePool);

#[derive(Serialize, Deserialize)]
pub struct OtpResponse {
    success: bool,
    message: String,
}


#[tauri::command]
pub async fn request_password_reset_otp(
    email: String,
    state: State<'_, DbState>,
) -> Result<OtpResponse, String> {
    let pool = &state.0;
    let now = Utc::now();


    let last_req: Option<chrono::DateTime<Utc>> = sqlx::query_scalar(
        "SELECT last_requested_at FROM otp_code WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(last_time) = last_req {
        if now - last_time < Duration::seconds(30) {
            return Ok(OtpResponse {
                success: false,
                message: "Подождите 30 секунд перед повторным запросом".to_string(),
            });
        }
    }

    let code: u32 = rand::rng().random_range(100_000..999_999);
    let code_str = code.to_string();
    let expires_at = now + Duration::minutes(5);

    sqlx::query(
        r#"
        INSERT INTO otp_code (email, code, expires_at, attempts, last_requested_at)
        VALUES (?1, ?2, ?3, 0, ?4)
        ON CONFLICT(email) DO UPDATE SET
            code = ?2,
            expires_at = ?3,
            attempts = 0,
            last_requested_at = ?4
        "#
    )
    .bind(&email)
    .bind(&code_str)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;


    match send_workout_report(email.clone(), code_str).await {
        Ok(msg) => {
            Ok(OtpResponse {
                success: true,
                message: "Код подтверждения отправлен на вашу почту".to_string(),
            })
        },
        Err(err) => {
            error!("{}", err);
            Ok(OtpResponse {
                success: false,
                message: "Не удалось отправить код на почту".to_string(),
            })
        }
            
    }
}

pub struct PasswordResetSession(pub std::sync::Mutex<Option<String>>);


#[tauri::command]
pub async fn verify_password_reset_otp(
    email: String,
    code: String,
    session: State<'_, PasswordResetSession>,
) -> Result<OtpResponse, String> {
    let pool = &state.0;
    let now = Utc::now();

    let row = sqlx::query!(
        "SELECT code, expires_at, attempts FROM otp_code WHERE email = ?",
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let otp = match row {
        Some(r) => r,
        None => return Ok(
            OtpResponse { 
                success: false, 
                message: "Код не запрашивался".to_string(), 
            }
        ),
    };

    let fail_string: Option<String> = None;
    if otp.attempts >= 3{
        fail_string = Some("Превышено число попыток. Запросите код заново".to_string());
    }
    if !fail_string.is_some()  && now > otp.expires_at.parse().unwrap_or(now){
        fail_string = Some("Срок действия кода истек".to_string());
    }


    if fail_string.is_some() {
        sqlx::query("DELETE FROM otp_code WHERE email = ?")
            .bind(&email)
            .execute(pool)
            .await
            .ok();
        return Ok(
            OtpResponse { 
                success: false, 
                message: fail_string.unwrap(), 
            }
        );
    }


    if otp.code == code {
        sqlx::query("DELETE FROM otp_code WHERE email = ?")
            .bind(&email)
            .execute(pool)
            .await
            .ok();

            *session.0.lock().unwrap() = Some(email);
            return Ok(OtpResponse { 
                success: true, 
                message: "Код подтвержден".to_string(), 
            }); 
    } else {
        sqlx::query("UPDATE otp_code SET attempts = attempts + 1 WHERE email = ?")
            .bind(&email)
            .execute(pool)
            .await
            .ok();

        Ok(OtpResponse {
            success: false,
            message: "Неверный код подтверждения".to_string(),
        })
    }
}


#[tauri::command]
pub async fn update_user_password(
    new_password: String, 
    session: tauri::State<'_, PasswordResetSession>,
    db_state: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let mut lock = session.0.lock().unwrap();
    let email = match lock.take() { 
        Some(e) => e,
        None => return Err("Действие запрещено. Вы не прошли проверку OTP".to_string()),
    };

    let pool = &db_state.0;
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(new_password.as_bytes())
        .map_err(|e| e.to_string())? 
        .to_string();

    sqlx::query("UPDATE user SET password_hash = ? WHERE email = ?")
        .bind(&password_hash)
        .bind(&email)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Пароль успешно изменен".to_string())
}