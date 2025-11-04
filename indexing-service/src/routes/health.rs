use crate::models::responses::HealthResponse;
use axum::response::Json;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "indexing-service".to_string(),
        status: "running".to_string(),
    })
}