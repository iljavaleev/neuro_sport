use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2
};


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