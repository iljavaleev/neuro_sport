use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2
};
use tauri::State;
use crate::state::AppState;
use crate::types::{FrontendUser, UserResponse};


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