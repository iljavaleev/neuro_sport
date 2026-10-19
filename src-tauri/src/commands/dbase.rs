use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn save_workout_results(
    state: State<'_, AppState>,
    session_id: i64,
    duration_seconds: f64,
    completed_rounds: i64,
    metrics: serde_json::Value,
) -> Result<String, String> {
    
    let json_wrapper = sqlx::types::Json(metrics);

    sqlx::query!(
        r#"
        INSERT INTO exercise_stats (session_id, duration_seconds, completed_rounds, dynamic_metrics)
        VALUES (?, ?, ?, ?)
        "#,
        session_id,
        duration_seconds,
        completed_rounds,
        json_wrapper
    )
    .execute(&state.db)
    .await
    .map_err(|e| format!("Ошибка сохранения метрик в БД: {}", e))?;

    Ok("Результаты тренировки успешно сохранены!".to_string())
}


#[tauri::command]
pub async fn get_workout_history_by_user(
    state: State<'_, AppState>,
    user_id: i64
) -> Result<Vec<ExerciseStat>, String> {
    
    let history = sqlx::query_as!(
        ExerciseStat,
        r#"
        SELECT es.id, es.session_id, es.duration_seconds, es.completed_rounds, es.dynamic_metrics
        FROM exercise_stats es
        JOIN sessions s ON es.session_id = s.id
        WHERE s.user_id = ?
        ORDER BY s.session_date DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Не удалось загрузить историю: {}", e))?;

    Ok(history)
}