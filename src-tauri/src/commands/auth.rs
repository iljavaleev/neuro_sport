use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2
};
use log::{info, error};
use tauri::State;
use crate::state::AppState;
use crate::types::{FrontendUser, UserResponse, OtpResponse, DbState, UserSessionData};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rand::RngExt;
use chrono::{Utc, Duration};
use base64::{prelude::BASE64_STANDARD, Engine};

use uuid::Uuid;


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


// https://mailtrap.io/sandboxes/4998701/settings
use lettre::transport::smtp::client::{Tls, TlsParameters};

#[tauri::command]
async fn send_otp_code(recipient_email: String, body_text: String) -> Result<String, String> {
    let email = Message::builder()
        .from("Sport Timer App <no-reply@iljavaleev.com>".parse().unwrap())
        .to(recipient_email.parse().map_err(|e| format!("Invalid recipient: {e}"))?)
        .subject("Код для смены пароля")
        .body(body_text)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(
        "9ff9ba3764b2e1".to_string(), 
        "fa8f3aa6b0ce98".to_string()
    );

    let tls_parameters = TlsParameters::builder("sandbox.smtp.mailtrap.io".to_string())
        .build_rustls()
        .map_err(|e| format!("TLS Initialization Error: {e}"))?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("sandbox.smtp.mailtrap.io")
        .map_err(|e| e.to_string())?
        .port(2525)
        .tls(Tls::Required(tls_parameters))
        .credentials(creds)
        .build();


    match mailer.send(email).await {
        Ok(_) => Ok("Отправлено".to_string()),
        Err(err) => Err(format!("Ошибка при отправке письма: {err}")),
    }
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
                code: None
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


    match send_otp_code(email.clone(), code_str).await {
        Ok(_) => {
            Ok(OtpResponse {
                success: true,
                message: "Код подтверждения отправлен на вашу почту".to_string(),
                code: None
            })
        },
        Err(err) => {
            error!("{}", err);
            Ok(OtpResponse {
                success: false,
                message: "Не удалось отправить код на почту".to_string(),
                code: None
            })
        }
            
    }
}

#[tauri::command]
pub async fn verify_password_reset_otp(
    email: String,
    code: String,
    state: State<'_, DbState>,
) -> Result<OtpResponse, String> {
    let pool = &state.0;
    let now = Utc::now().naive_utc();

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
                code: None 
            }
        ),
    };

    let mut fail_string: Option<String> = None;
    if otp.attempts.unwrap() >= 3{
        fail_string = Some("Превышено число попыток. Запросите код заново".to_string());
    }
    if !fail_string.is_some() && (now > otp.expires_at){
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
                code: None 
            }
        );
    }


    if otp.code == code {
        sqlx::query("DELETE FROM otp_code WHERE email = ?")
            .bind(&email)
            .execute(pool)
            .await
            .ok();

        let reset_expires_at = Utc::now() + chrono::Duration::minutes(10);
    
        let raw_token = format!("{}|{}", email, reset_expires_at.to_rfc3339());
        
        let reset_token = BASE64_STANDARD.encode(raw_token);

        return Ok(OtpResponse { 
            success: true, 
            message: "Код подтвержден".to_string(),
            code: Some(reset_token)
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
            code: None
        })
    }
}

#[tauri::command]
pub async fn update_user_password(
    code: String,
    new_password: String, 
    db_state: tauri::State<'_, DbState>,
) -> Result<String, String> {
   
    let decoded_bytes = BASE64_STANDARD.decode(code)
        .map_err(|_| "Невалидный токен сброса".to_string())?;
    let decoded_str = String::from_utf8(decoded_bytes)
        .map_err(|_| "Ошибка чтения данных токена".to_string())?;

    let parts: Vec<&str> = decoded_str.split('|').collect();
    if parts.len() != 2 {
        return Err("Неверный формат токена".to_string());
    }

    let email = parts[0];
    let expires_at_str = parts[1];

    let expires_at = chrono::DateTime::parse_from_rfc3339(expires_at_str)
        .map_err(|_| "Ошибка парсинга времени токена".to_string())?
        .with_timezone(&Utc);

    if Utc::now() > expires_at {
        return Err("Время действия ссылки на сброс пароля истекло".to_string());
    }


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



#[tauri::command]
pub async fn check_user_password(
    id: String,
    passwd: String,
    state: State<'_, DbState>,
) -> Result<OtpResponse, String> {
    
    let row = sqlx::query!(
        "SELECT password_hash FROM user WHERE id = ?",
        id
    )
    .fetch_optional(&state.0)
    .await
    .map_err(|e| format!("Database error: {e}"))?;
    
    let db_hash = match row {
        Some(r) => r.password_hash,
        None => {
            return Ok(OtpResponse {
                success: false,
                message: "Пользователь не найден".to_string(),
                code: None,
                auth_info: None,
            })
        }
    };

    let parsed_hash = PasswordHash::new(&db_hash)
        .map_err(|e| format!("Invalid password hash format in DB: {e}"))?;

    match Argon2::default().verify_password(passwd.as_bytes(), &parsed_hash) {
        Ok(_) => {

            let token = Uuid::new_v4().to_string();

            let expires_duration = Duration::days(14);
            let expires_at_ms = (Utc::now() + expires_duration).timestamp_millis() as f64;

            sqlx::query!(
                "INSERT INTO user_session (token, user_id, expires_at) VALUES (?, ?, ?)",
                token,
                id,
                expires_at_ms
            )
            .execute(&state.0)
            .await
            .map_err(|e| format!("Failed to save auth token: {e}"))?;

            Ok(OtpResponse {
                success: true,
                message: "Пароль верный".to_string(),
                code: None,
                auth_info: Some(UserSessionData {
                    user_id: id,
                    token,
                    expires_at: expires_at_ms,
                }),
            })
        },
        Err(_) => Ok(OtpResponse {
            success: false,
            message: "Неверный пароль".to_string(),
            code: None,
            auth_info: None,
        }),
    }
}


#[tauri::command]
pub async fn verify_user_token(
    token: String,
    state: State<'_, DbState>,
) -> Result<Option<UserSessionData>, String> {
    let now_ms = Utc::now().timestamp_millis() as f64;

    let row = sqlx::query!(
        "SELECT user_id, expires_at FROM user_session WHERE token = ? AND expires_at > ?",
        token,
        now_ms
    )
    .fetch_optional(&state.0)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    match row {
        Some(r) => Ok(Some(UserSessionData {
            user_id: r.user_id,
            token,
            expires_at: r.expires_at,
        })),
        None => {
            let _ = sqlx::query!("DELETE FROM user_session WHERE token = ?", token)
                .execute(&state.0)
                .await;
            Ok(None)
        }
    }
}

