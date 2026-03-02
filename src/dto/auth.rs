use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Входные данные для регистрации.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "mypassword123")]
    pub password: String,
}

/// Входные данные для логина (авторизации).
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "mypassword123")]
    pub password: String,
}

/// Ответ на успешную авторизацию / регистрацию.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT-токен для авторизации последующих запросов.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
}

/// Ответ на GET /auth/me — информация о текущем пользователе.
#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "2026-03-03T12:00:00Z")]
    pub created_at: Option<String>,
}
